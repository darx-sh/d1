/*
  Warnings:

  - You are about to drop the column `status` on the `Deployment` table. All the data in the column will be lost.

*/
-- AlterTable
ALTER TABLE `Deployment` DROP COLUMN `status`,
    ADD COLUMN `funcUploadCnt` INTEGER NOT NULL DEFAULT 0,
    ADD COLUMN `uploadStatus` VARCHAR(191) NOT NULL DEFAULT 'running';

-- AlterTable
ALTER TABLE `Function` ADD COLUMN `uploadStatus` VARCHAR(191) NOT NULL DEFAULT 'running';
