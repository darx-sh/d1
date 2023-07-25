import { S3Client } from "@aws-sdk/client-s3";
import { env } from "~/env.mjs";

const globalForS3 = globalThis as unknown as {
  s3: S3Client | undefined;
};

// const s3 =
//   globalForS3.s3 ??
//   new S3Client({
//     region: env.S3_REGION,
//     credentials: {
//       accessKeyId: env.S3_ACCESS_KEY_ID,
//       secretAccessKey: env.S3_SECRET_ACCESS_KEY,
//     },
//   });
//
// if (env.NODE_ENV !== "production") globalForS3.s3 = s3;

// export default s3;
