import React, { useState } from "react";
import { useRouter } from "next/router";
import TopNav from "~/components/project/TopNav";
import LeftContainer from "~/components/project/LeftContainer";
import RightContainer from "~/components/project/RightContainer";
import { ProjectProvider } from "~/components/project/ProjectContext";

function ProjectDetail() {
  const router = useRouter();
  const projectId = router.query.id as string;
  const topNav = [
    { name: "Home", href: "/" },
    { name: "Projects", href: "/projects" },
    { name: `${projectId}`, href: "#" },
  ];

  return (
    <ProjectProvider>
      <div className="flex h-screen flex-col bg-gray-100">
        <div className="h-16">
          <TopNav nav={topNav}></TopNav>
        </div>
        <div className="flex flex-1 space-x-2">
          <div className="w-56 border-r-2 border-t-2 border-gray-300 bg-gray-50">
            <LeftContainer></LeftContainer>
          </div>
          <div className="mb-5 flex-1 rounded border-2 border-gray-300 bg-white shadow-lg">
            <RightContainer></RightContainer>
          </div>
        </div>
      </div>
    </ProjectProvider>
  );
}

export default ProjectDetail;
