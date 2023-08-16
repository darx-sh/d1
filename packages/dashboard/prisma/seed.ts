import { PrismaClient } from "@prisma/client";

const prisma = new PrismaClient();

async function main() {
  if (process.env.NODE_ENV === "development") {
    // create one project with one environment
    const proj = await prisma.project.upsert({
      where: { id: "cku0q1q6h0000h1t9q6q1q6h0" },
      update: {},
      create: {
        name: "test project",
        // fake organization
        organizationId: "fake",
        environments: {
          create: [{ id: "8nvcym53y8d2", name: "dev" }],
        },
      },
    });
  }
}

main()
  .then(async () => {
    await prisma.$disconnect();
  })
  .catch(async (e) => {
    console.error(e);
    await prisma.$disconnect();
    process.exit(1);
  });
