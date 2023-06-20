/*
  Warnings:

  - You are about to drop the column `funcUploadCnt` on the `Deployment` table. All the data in the column will be lost.
  - You are about to drop the `Function` table. If the table is not empty, all the data it contains will be lost.

*/
-- AlterTable
ALTER TABLE `Deployment` DROP COLUMN `funcUploadCnt`,
    ADD COLUMN `bundleUploadCnt` INTEGER NOT NULL DEFAULT 0;

-- DropTable
DROP TABLE `Function`;

-- CreateTable
CREATE TABLE `Bundle` (
    `id` VARCHAR(191) NOT NULL,
    `createdAt` DATETIME(3) NOT NULL DEFAULT CURRENT_TIMESTAMP(3),
    `updatedAt` DATETIME(3) NOT NULL,
    `path` TEXT NOT NULL,
    `bytes` INTEGER NOT NULL,
    `uploadStatus` VARCHAR(191) NOT NULL DEFAULT 'running',
    `deploymentId` VARCHAR(191) NOT NULL,

    INDEX `Bundle_deploymentId_idx`(`deploymentId`),
    PRIMARY KEY (`id`)
) DEFAULT CHARACTER SET utf8mb4 COLLATE utf8mb4_unicode_ci;
