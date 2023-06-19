import { NextRequest, NextResponse } from "next/server";
import { prisma } from "~/server/db";

export async function POST(req: NextRequest) {
  let data = await req.json();
  return NextResponse.json({ data: data });
}
