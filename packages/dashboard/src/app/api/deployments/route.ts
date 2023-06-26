import { getSignedUrl } from "@aws-sdk/s3-request-presigner";
import { PutObjectCommand } from "@aws-sdk/client-s3";

import { NextRequest, NextResponse } from "next/server";
import { env } from "~/env.mjs";
import { prisma } from "~/server/db";
import s3 from "~/server/s3";
import { PRESIGNED_URL_EXPIRES_SECONDS } from "~/server/constants";

type PrepareDeploymentReq = {
  environment_id: string;
  tag: string | null;
  description: string | null;
  bundles: BundleReq[];
  metas: BundleMeta[];
};

type PrepareDeploymentRsp = {
  deployment_id: string;
  bundles: BundleRsp[];
};

type BundleReq = {
  path: string;
  bytes: number;
  checksum: string;
  checksum_type: string;
};

type BundleRsp = {
  id: string;
  path: string;
  upload_url: string;
};

type BundleMeta = {
  entry_point: string;
  exports: [string];
};

type HttpRoute = {
  path: string;
  method: string;
  jsEntryPoint: string;
  jsExport: string;
};

// prepare a deployment
export async function POST(req: NextRequest) {
  const prepare_req = (await req.json()) as PrepareDeploymentReq;
  const { environment_id, tag, description, bundles, metas } = prepare_req;
  const theEnv = await prisma.environment.findFirst({
    where: {
      id: environment_id,
    },
  });

  if (!theEnv) {
    return NextResponse.json(
      { error: `Environment Not found: ${environment_id}` },
      { status: 400 }
    );
  }

  const bundlesData = bundles.map((bundle) => {
    return {
      path: bundle.path,
      bytes: bundle.bytes,
    };
  });
  const bundleCount = bundles.length;
  const routes = metas
    .map((meta) => {
      const routes = meta.exports.map((jsExport) => {
        const route: HttpRoute = {
          path: buildHttpRoute(meta.entry_point, jsExport),
          method: "POST",
          jsEntryPoint: meta.entry_point,
          jsExport: jsExport,
        };
        return route;
      });
      return routes;
    })
    .flat();
  const deployment = await prisma.deployment.create({
    data: {
      tag,
      description,
      environmentId: environment_id,
      bundleCount,
      bundles: {
        create: bundlesData,
      },
      httpRoutes: {
        create: routes,
      },
    },
    include: {
      bundles: true,
    },
  });

  const urlPromises = deployment.bundles.map((bundle) => {
    const putCommand = new PutObjectCommand({
      Bucket: env.S3_BUCKET,
      Key: `${deployment.id}/${bundle.id}`,
      ContentLength: bundle.bytes,
    });
    return getSignedUrl(s3, putCommand, {
      expiresIn: PRESIGNED_URL_EXPIRES_SECONDS,
    });
  });

  const urls = await Promise.all(urlPromises);
  const rsp: PrepareDeploymentRsp = {
    deployment_id: deployment.id,
    bundles: [],
  };

  for (let i = 0; i < urls.length; i++) {
    const bundle = deployment.bundles[i];
    const url = urls[i];
    if (bundle !== undefined && url !== undefined) {
      rsp.bundles.push({
        id: bundle.id,
        path: bundle.path,
        upload_url: url,
      });
    } else {
      if (bundle === undefined && url === undefined) {
        throw new Error(
          `failed to prepare deployment: deploymentId = ${deployment.id}, i = ${i}, bundle undefined, url undefined`
        );
      } else if (url === undefined) {
        throw new Error(
          `failed to prepare deployment: deploymentId = ${deployment.id}, i = ${i}, url undefined`
        );
      } else if (bundle === undefined) {
        throw new Error(
          `failed to prepare deployment: deploymentId = ${deployment.id}, i = ${i}, bundle undefined`
        );
      }
    }
  }
  return NextResponse.json(rsp);
}

function buildHttpRoute(jsEntryPoint: string, jsExport: string) {
  let path: string;
  if (
    jsEntryPoint.endsWith(".js") ||
    jsEntryPoint.endsWith(".ts") ||
    jsEntryPoint.endsWith(".mjs")
  ) {
    path = jsEntryPoint.slice(0, jsEntryPoint.lastIndexOf("."));
  } else {
    throw new Error("invalid meta");
  }

  if (jsExport !== "default") {
    path = `${path}.${jsExport}`;
  }
  return path;
}
