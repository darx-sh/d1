-- CreateTable
CREATE TABLE `HttpRoute` (
    `id` VARCHAR(191) NOT NULL,
    `createdAt` DATETIME(3) NOT NULL DEFAULT CURRENT_TIMESTAMP(3),
    `updatedAt` DATETIME(3) NOT NULL,
    `path` VARCHAR(191) NOT NULL,
    `method` VARCHAR(191) NOT NULL,
    `jsEntryPoint` VARCHAR(191) NOT NULL,
    `jsExport` VARCHAR(191) NOT NULL,
    `deploymentId` VARCHAR(191) NOT NULL,

    INDEX `HttpRoute_deploymentId_idx`(`deploymentId`),
    PRIMARY KEY (`id`)
) DEFAULT CHARACTER SET utf8mb4 COLLATE utf8mb4_unicode_ci;
