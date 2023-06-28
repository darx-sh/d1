import { NextRequest, NextResponse } from "next/server";
import { prisma } from "~/server/db";
import redis from "~/server/redis";
import { UPLOAD_FAILED, UPLOAD_SUCCESS } from "~/server/constants";

// update deployment status
export async function POST(
  req: NextRequest,
  { params }: { params: { deploy_id: string; bundle_id: string } }
) {
  const { status } = (await req.json()) as { status: string };
  if (status === "success") {
    await success_upload(params.deploy_id, params.bundle_id);
  } else if (status === "failed") {
    await fail_upload(params.deploy_id, params.bundle_id);
  } else {
    return NextResponse.json(
      { error: `Invalid status: ${status}` },
      { status: 400 }
    );
  }
  return NextResponse.json({}, { status: 200 });
}

export function GET() {
  return NextResponse.json("ok");
}

async function success_upload(deploy_id: string, func_id: string) {
  const [_, deploy] = await prisma.$transaction([
    prisma.bundle.update({
      where: { id: func_id },
      data: { uploadStatus: UPLOAD_SUCCESS },
    }),
    prisma.deployment.update({
      where: { id: deploy_id },
      data: {
        bundleUploadCnt: {
          increment: 1,
        },
      },
      include: {
        bundles: true,
        environment: true,
        httpRoutes: true,
      },
    }),
  ]);

  if (deploy.bundleUploadCnt === deploy.bundleCount) {
    // todo: we might considering the following:
    // 1. add a auto increment field to `Deployment` model.
    // 2. use redis stream to publish the deploy_id, and use the above sequence for the stream id.
    // await redis.xadd("deploy", "*", "deploy_id", deploy.id);
    const bundles = deploy.bundles.map((bundle) => {
      return {
        id: bundle.id,
        fs_path: bundle.fsPath,
      };
    });
    const httpRoutes = deploy.httpRoutes.map((route) => {
      return {
        http_path: route.httpPath,
        method: route.method,
        js_entry_point: route.jsEntryPoint,
        js_export: route.jsExport,
      };
    });
    await redis.publish(
      "deploy",
      JSON.stringify({
        project_id: deploy.environment.projectId,
        environment_id: deploy.environmentId,
        deployment_id: deploy.id,
        deploy_seq: deploy.deploySeq,
        bundles: bundles,
        http_routes: httpRoutes,
      })
    );
  }
}

async function fail_upload(deploy_id: string, func_id: string) {
  return prisma.bundle.update({
    where: { id: func_id },
    data: { uploadStatus: UPLOAD_FAILED },
  });
}
