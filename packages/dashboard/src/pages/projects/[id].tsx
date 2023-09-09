import React, { useEffect, useState } from "react";
import { useRouter } from "next/router";
import TopNav from "~/components/project/TopNav";
import LeftContainer from "~/components/project/LeftContainer";
import RightContainer from "~/components/project/RightContainer";
import Spinner from "~/components/project/Spinner";
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
import { DatabaseProvider } from "~/components/project/database/DatabaseContext";

function ProjectDetail() {
  const router = useRouter();
  const { query } = router;
  const [isLoading, setIsLoading] = useState(true);
  const projectDispatch = useProjectDispatch();
  const projectState = useProjectState();
  type HttpRoute = {
    http_path: string;
    method: string;
    js_entry_point: string;
    js_export: string;
  };

  useEffect(() => {
    type LoadEnvRsp = {
      codes: { fs_path: string; content: string }[];
      http_routes: HttpRoute[];
      project: { id: string; name: string };
      env: { id: string; name: string };
    };

    if (!router.isReady) {
      return;
    }

    const projectId = query.id as string;
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
  }, [router.isReady]);

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
        <>
          <p>{query.id}</p>
          <Spinner></Spinner>
        </>
      ) : (
        <>
          <div className="fixed left-0 right-0 top-0 h-14 bg-gray-100">
            <TopNav></TopNav>
          </div>
          <div className="fixed bottom-0 left-0 top-14 w-48 bg-gray-100">
            <LeftContainer></LeftContainer>
          </div>
          <div className="fixed bottom-0 left-48 right-0 top-14 min-w-0  border bg-white">
            <RightContainer></RightContainer>
          </div>
        </>
      )}
    </>
  );
}

function ProjectDetailWrapper() {
  return (
    <ProjectProvider>
      <DatabaseProvider>
        <ProjectDetail></ProjectDetail>
      </DatabaseProvider>
    </ProjectProvider>
  );
}

export default ProjectDetailWrapper;
