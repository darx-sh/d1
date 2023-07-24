import React, { useEffect, useRef, useState } from "react";
import FileMenu from "~/components/project/FileMenu";
import TreeView, { NodeId } from "~/components/react-tree-view";
import {
  FolderIcon,
  FolderOpenIcon,
  DocumentPlusIcon,
} from "@heroicons/react/24/solid";

import {
  useProjectState,
  useProjectDispatch,
  CodeChecksums,
} from "~/components/project_v2/ProjectContext";

import axios from "axios";
import axiosRetry from "axios-retry";
import { useInterval } from "usehooks-ts";
import { ITreeViewOnNodeSelectProps } from "~/components/react-tree-view";

type MenuPosition = {
  coord: { x: number; y: number } | null;
  nodeParent: NodeId | null;
  nodeIsBranch: boolean | undefined;
  currentNodeId: NodeId;
};

export default function LeftDirectory() {
  const [expandedIds, setExpandedIds] = useState<NodeId[]>(["/functions"]);
  const [isLoading, setIsLoading] = useState(true);
  const [menuPosition, setMenuPosition] = useState({
    coord: null,
    nodeParent: null,
    nodeIsBranch: undefined,
    currentNodeId: "",
  } as MenuPosition);

  const [newFileParent, setNewFileParent] = useState<NodeId | null>(null);
  const newFileInputRef = useRef<HTMLInputElement>(null);

  const handleNewFileCreate = (
    event: React.KeyboardEvent<HTMLInputElement>
  ) => {
    if (event.key === "Enter") {
      event.preventDefault();
      const inputValue = newFileInputRef.current!.value;
      if (inputValue.endsWith(".js") || inputValue.endsWith(".ts")) {
        projectDispatch!({
          type: "NewJsFile",
          parentNodeId: newFileParent!,
          fileName: inputValue,
        });
        setNewFileParent(null);
      } else {
        alert("only js or ts file supported");
      }
    }
  };

  // focus on the input field when creating a new file.
  // runs every time isCreatingNewFile changes.
  useEffect(() => {
    if (newFileInputRef.current) {
      newFileInputRef.current.focus();
    }
  });

  const handleContextMenu = (
    event: React.MouseEvent<Element, MouseEvent>,
    nodeParent: NodeId | null,
    nodeIsBranch: boolean | undefined,
    currentNodeId: NodeId
  ) => {
    event.preventDefault();
    setMenuPosition({
      coord: { x: event.clientX, y: event.clientY },
      nodeParent,
      nodeIsBranch,
      currentNodeId,
    });
  };

  const projectState = useProjectState();
  const projectDispatch = useProjectDispatch();

  type HttpRoute = {
    js_entry_point: string;
    js_export: string;
    http_path: string;
    method: string;
  };

  // load code from server
  useEffect(() => {
    type ListCodeRsp = {
      codes: { fs_path: string; content: string }[];
      http_routes: HttpRoute[];
    };
    const listCodeUrl =
      "http://localhost:3457/list_code/cljb3ovlt0002e38vwo0xi5ge";
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
          };
        });
        projectDispatch!({
          type: "LoadCodes",
          codes: codeState,
          httpRoutes: routeState,
        });
        setIsLoading(false);
      })
      .catch((error) => console.error("list_code error: ", error));
  }, []);

  const [deployingCode, setDeployingCode] = useState(false);
  useInterval(() => {
    type DeployCodeRsp = {
      http_routes: HttpRoute[];
    };

    if (deployingCode) {
      return;
    }

    let codeChanged = false;
    const codes = projectState!.directory.codes;
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
      const deployCodeUrl =
        "http://localhost:3457/deploy_code/cljb3ovlt0002e38vwo0xi5ge";
      const codReq = codes.map((c) => {
        return { fs_path: c.fsPath, content: c.content };
      });
      axios
        .post(
          deployCodeUrl,
          { codes: codReq },
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
            };
          });
          setDeployingCode(false);
          projectDispatch!({
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

  const handleNodeSelect = (nodeSelectProps: ITreeViewOnNodeSelectProps) => {
    const { isBranch, element } = nodeSelectProps;
    if (!isBranch) {
      projectDispatch!({
        type: "OpenJsFile",
        nodeId: element.id,
      });
    }
  };

  return (
    <>
      {isLoading ? (
        renderLoadingBar()
      ) : (
        <>
          <div>
            {menuPosition.coord && (
              <FileMenu
                menuPosition={menuPosition.coord}
                hideMenu={() =>
                  setMenuPosition({ ...menuPosition, coord: null })
                }
                handleNewFile={() => {
                  let parentId: NodeId | null = null;
                  if (menuPosition.nodeIsBranch) {
                    parentId = menuPosition.currentNodeId;
                  } else {
                    parentId = menuPosition.nodeParent;
                  }
                  setNewFileParent(parentId);
                  if (
                    undefined === expandedIds.find((id) => id === parentId) &&
                    parentId !== null
                  ) {
                    setExpandedIds([...expandedIds, parentId]);
                  }
                }}
              ></FileMenu>
            )}
          </div>
          <TreeView
            data={projectState!.directory.treeViewData}
            defaultExpandedIds={["/functions", "/"]}
            expandedIds={expandedIds}
            onNodeSelect={handleNodeSelect}
            aria-label="directory tree"
            nodeRenderer={({
              element,
              isSelected,
              isBranch,
              isExpanded,
              getNodeProps,
              level,
            }) => {
              return (
                <div className="mt-4 pl-3">
                  <div
                    {...getNodeProps()}
                    style={{ paddingLeft: 20 * (level - 1) }}
                    onContextMenu={(event) => {
                      handleContextMenu(
                        event,
                        element.parent,
                        element.isBranch,
                        element.id
                      );
                    }}
                  >
                    {isBranch && isExpanded ? (
                      <FolderOpenIcon className="mr-1 h-4 w-4" />
                    ) : null}
                    {isBranch && !isExpanded ? (
                      <FolderIcon className="mr-1 h-4 w-4" />
                    ) : null}
                    <span className="-mb-0.5">{element.name}</span>
                  </div>
                  {newFileParent != null &&
                  isBranch &&
                  newFileParent === element.id ? (
                    <div className="ml-5 mt-2">
                      <input
                        type="text"
                        id="new-file-name"
                        name="new-file-name"
                        ref={newFileInputRef}
                        className="h-6 w-40 rounded px-1.5 text-gray-900 shadow-sm outline-1 placeholder:text-gray-400"
                        placeholder="New File"
                        onKeyDown={handleNewFileCreate}
                        onBlur={(event) => {
                          setNewFileParent(null);
                        }}
                      />
                    </div>
                  ) : null}
                </div>
              );
            }}
          />
        </>
      )}
    </>
  );
}
