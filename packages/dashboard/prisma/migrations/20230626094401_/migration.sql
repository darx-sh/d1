/*
  Warnings:

  - You are about to drop the column `path` on the `Bundle` table. All the data in the column will be lost.
  - You are about to drop the column `path` on the `HttpRoute` table. All the data in the column will be lost.
  - Added the required column `fsPath` to the `Bundle` table without a default value. This is not possible if the table is not empty.
  - Added the required column `httpPath` to the `HttpRoute` table without a default value. This is not possible if the table is not empty.

*/
-- AlterTable
ALTER TABLE `Bundle` DROP COLUMN `path`,
    ADD COLUMN `fsPath` TEXT NOT NULL;

-- AlterTable
ALTER TABLE `HttpRoute` DROP COLUMN `path`,
    ADD COLUMN `httpPath` VARCHAR(191) NOT NULL;
