import React, { useEffect, useState } from "react";
import { useRouter } from "next/router";
import TopNav from "~/components/project/TopNav";
import LeftContainer from "~/components/project/LeftContainer";
import RightContainer from "~/components/project/RightContainer";
import {
  CodeChecksums,
  initialHttpParam,
  ProjectProvider,
  useProjectDispatch,
  useProjectState,
} from "~/components/project/ProjectContext";
import axios from "axios";
import axiosRetry from "axios-retry";
import { useInterval } from "usehooks-ts";

function ProjectDetail() {
  const router = useRouter();
  const projectId = router.query.id as string;
  const topNav = [
    { name: "Home", href: "/" },
    { name: "Projects", href: "/projects" },
    { name: `${projectId}`, href: "#" },
  ];
  const [isLoading, setIsLoading] = useState(true);
  const projectDispatch = useProjectDispatch()!;
  const projectState = useProjectState()!;

  type HttpRoute = {
    http_path: string;
    method: string;
    js_entry_point: string;
    js_export: string;
  };

  useEffect(() => {
    type ListCodeRsp = {
      codes: { fs_path: string; content: string }[];
      http_routes: HttpRoute[];
    };
    const listCodeUrl = "http://localhost:3457/list_code/8nvcym53y8d2";
    const instance = axios.create();
    axiosRetry(instance, {
      retries: 100,
      shouldResetTimeout: true,
      retryDelay: (retryCount) => {
        return 1000;
      },
    });
    instance
      .get(listCodeUrl, {
        timeout: 4000,
      })
      .then((response) => {
        const { codes, http_routes } = response.data as ListCodeRsp;
        const codeState = codes.map((code) => {
          return {
            fsPath: code.fs_path,
            content: code.content,
          };
        });
        const routeState = http_routes.map((route) => {
          return {
            jsEntryPoint: route.js_entry_point,
            jsExport: route.js_export,
            httpPath: route.http_path,
            method: route.method,
            curParams: initialHttpParam,
          };
        });
        projectDispatch({
          type: "LoadCodes",
          codes: codeState,
          httpRoutes: routeState,
        });
        setIsLoading(false);
      })
      .catch((error) => console.error("list_code error: ", error));
  }, []);

  const renderLoadingBar = () => {
    return (
      <svg
        className="flex h-10 w-10 animate-spin items-center text-gray-500"
        viewBox="0 0 24 24"
      >
        <circle
          className="opacity-25"
          cx="12"
          cy="12"
          r="10"
          stroke="currentColor"
          strokeWidth="4"
        />
        <path
          className="opacity-75"
          fill="currentColor"
          d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm12 0a8 8 0 018 8V0c0-6.627-5.373-12-12-12h4zm-4 4a4 4 0 100 8 4 4 0 000-8zm0 6a2 2 0 110-4 2 2 0 010 4z"
        />
      </svg>
    );
  };

  const [deployingCode, setDeployingCode] = useState(false);
  useInterval(() => {
    type DeployCodeRsp = {
      http_routes: HttpRoute[];
    };

    if (deployingCode) {
      return;
    }

    let codeChanged = false;
    const codes = projectState.directory.codes;
    const curChecksums: CodeChecksums = codes
      .map((c) => {
        return { fsPath: c.fsPath, checksum: c.curChecksum };
      })
      .reduce<CodeChecksums>((result, item) => {
        result[item.fsPath] = item.checksum!;
        return result;
      }, {});

    codes.forEach((code) => {
      if (code.curChecksum !== code.prevChecksum) {
        codeChanged = true;
        return;
      }
    });

    if (codeChanged) {
      setDeployingCode(true);
      // save code to server
      const deployCodeUrl = "http://localhost:3457/deploy_code/8nvcym53y8d2";
      const codReq = codes.map((c) => {
        return { fs_path: c.fsPath, content: c.content };
      });
      axios
        .post(
          deployCodeUrl,
          { codes: codReq, vars: [] },
          { timeout: 2000, headers: { "Content-Type": "application/json" } }
        )
        .then((response) => {
          const { http_routes } = response.data as DeployCodeRsp;
          const httpRoutes = http_routes.map((route) => {
            return {
              httpPath: route.http_path,
              method: route.method,
              jsEntryPoint: route.js_entry_point,
              jsExport: route.js_export,
              curParams: initialHttpParam,
            };
          });
          setDeployingCode(false);
          projectDispatch({
            type: "PersistedCode",
            checksums: curChecksums,
            httpRoutes,
          });
        })
        .catch((error) => {
          console.error("deploy_code error: ", error);
          setDeployingCode(false);
        });
    }
  }, 1000);

  return (
    <>
      {isLoading ? (
        renderLoadingBar()
      ) : (
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
      )}
    </>
  );
}

function ProjectContainer() {
  return (
    <ProjectProvider>
      <ProjectDetail></ProjectDetail>
    </ProjectProvider>
  );
}

export default ProjectContainer;
