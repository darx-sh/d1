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
    }),
  ]);
  await redis.xadd("deploy", "*", "deploy_id", deploy.id);
}

async function fail_upload(deploy_id: string, func_id: string) {
  return prisma.bundle.update({
    where: { id: func_id },
    data: { uploadStatus: UPLOAD_FAILED },
  });
}
