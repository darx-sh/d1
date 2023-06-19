import { NextRequest, NextResponse } from "next/server";
import { prisma } from "~/server/db";

export async function POST(req: NextRequest) {
  const data = (await req.json()) as { environmentId: string };
  return NextResponse.json({ data: data });
}
