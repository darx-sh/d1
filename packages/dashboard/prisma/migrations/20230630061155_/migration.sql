/*
  Warnings:

  - You are about to drop the column `bundleCount` on the `Deployment` table. All the data in the column will be lost.
  - Added the required column `bundleCnt` to the `Deployment` table without a default value. This is not possible if the table is not empty.

*/
-- AlterTable
ALTER TABLE `Deployment` DROP COLUMN `bundleCount`,
    ADD COLUMN `bundleCnt` INTEGER NOT NULL;
