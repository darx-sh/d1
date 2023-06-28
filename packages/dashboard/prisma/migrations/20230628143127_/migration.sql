/*
  Warnings:

  - A unique constraint covering the columns `[environmentId,deploySeq]` on the table `Deployment` will be added. If there are existing duplicate values, this will fail.

*/
-- DropIndex
DROP INDEX `Deployment_environmentId_idx` ON `Deployment`;

-- AlterTable
ALTER TABLE `Deployment` ADD COLUMN `deploySeq` INTEGER NOT NULL DEFAULT 0;

-- AlterTable
ALTER TABLE `Environment` ADD COLUMN `nextDeploySeq` INTEGER NOT NULL DEFAULT 0;

-- CreateIndex
CREATE UNIQUE INDEX `Deployment_environmentId_deploySeq_key` ON `Deployment`(`environmentId`, `deploySeq`);
