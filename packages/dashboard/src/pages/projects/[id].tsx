import React, { useEffect, useState } from "react";
import { useRouter } from "next/router";
import TopNav from "~/components/project/TopNav";
import LeftContainer from "~/components/project/LeftContainer";
import RightContainer from "~/components/project/RightContainer";
import LoadingBar from "~/components/project/LoadingBar";
import {
  CodeChecksums,
  initialHttpParam,
  ProjectProvider,
  useProjectDispatch,
  useProjectState,
} from "~/components/project/ProjectContext";
import axios from "axios";
import axiosRetry from "axios-retry";
import { useInterval, useEffectOnce } from "usehooks-ts";

function ProjectDetail() {
  const router = useRouter();
  const [isLoading, setIsLoading] = useState(true);
  const projectDispatch = useProjectDispatch()!;
  const projectState = useProjectState()!;
  type HttpRoute = {
    http_path: string;
    method: string;
    js_entry_point: string;
    js_export: string;
  };

  useEffectOnce(() => {
    type LoadEnvRsp = {
      codes: { fs_path: string; content: string }[];
      http_routes: HttpRoute[];
      project: { id: string; name: string };
      env: { id: string; name: string };
    };

    if (!router.isReady) {
      return;
    }

    const projectId = router.query.id as string;
    const loadEnvUrl = `http://localhost:3457/load_env/${projectId}`;
    const instance = axios.create();
    axiosRetry(instance, {
      retries: 1,
      shouldResetTimeout: true,
      retryDelay: (retryCount) => {
        return 1000;
      },
    });
    instance
      .get(loadEnvUrl, {
        timeout: 4000,
      })
      .then((response) => {
        const { codes, http_routes, project, env } =
          response.data as LoadEnvRsp;
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
          type: "LoadEnv",
          codes: codeState,
          httpRoutes: routeState,
          projectInfo: project,
          envInfo: env,
        });
        setIsLoading(false);
      })
      .catch((error) => console.error("load_env error: ", error));
  });

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
      const deployCodeUrl = `http://localhost:3457/deploy_code/${
        projectState.envInfo!.id
      }`;
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
        <LoadingBar></LoadingBar>
      ) : (
        <div className="flex h-screen flex-col bg-gray-100">
          <div className="h-16">
            <TopNav></TopNav>
          </div>
          <div className="flex flex-1 space-x-2">
            <div className="w-48 border-r-2 border-t-2 border-gray-300 bg-gray-50">
              <LeftContainer></LeftContainer>
            </div>
            <div className="mb-2 flex-1 rounded border-2 border-gray-300 bg-white shadow-lg">
              <RightContainer></RightContainer>
            </div>
          </div>
        </div>
      )}
    </>
  );
}

function ProjectDetailWrapper() {
  return (
    <ProjectProvider>
      <ProjectDetail></ProjectDetail>
    </ProjectProvider>
  );
}

export default ProjectDetailWrapper;
