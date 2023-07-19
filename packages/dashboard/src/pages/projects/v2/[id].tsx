import React, { useState } from "react";
import { useRouter } from "next/router";
import TopNav from "~/components/project_v2/TopNav";
import LeftContainer from "~/components/project_v2/LeftContainer";
import MiddleContainer from "~/components/project_v2/MiddleContainer";

function ProjectDetail() {
  const router = useRouter();
  const projectId = router.query.id as string;
  const topNav = [
    { name: "Home", href: "/" },
    { name: "Projects", href: "/projects" },
    { name: `${projectId}`, href: "#" },
  ];

  return (
    <div className="h-screen overflow-hidden">
      <div className="h-16 border">
        <TopNav nav={topNav}></TopNav>
      </div>
      <div className="flex h-full">
        <div className="w-56 border">
          <LeftContainer></LeftContainer>
        </div>
        <div className="flex-1 border">
          <MiddleContainer></MiddleContainer>
        </div>
        <div className="w-96 border"></div>
      </div>
    </div>
  );
}

export default ProjectDetail;
