import React, { useState, useEffect, useRef } from "react";
import { FaRegFolder, FaRegFolderOpen } from "react-icons/fa";
import TreeView, { flattenTree } from "~/components/react-tree-view";
import FileMenu from "./FileMenu";
import FileContent from "./FileContent";
import NewFileDialog from "./NewFileDialog";
import axios from "axios";

const data = [
  { id: "root", name: "", children: ["functions", "lib"], parent: null },
  {
    id: "functions",
    name: "functions",
    children: [],
    parent: "root",
    isBranch: true,
    metadata: { fsPath: "functions" },
  },
  {
    id: "lib",
    name: "lib",
    children: [],
    parent: "root",
    isBranch: true,
    metadata: { fsPath: "lib" },
  },
];

const defaultJsCode = `
export default function hello() {
  return "hello";
}
`;

function DirectoryTreeView() {
  const [isCodeLoading, setIsCodeLoading] = useState(true);
  const [isMenuOpen, setMenuOpen] = useState(false);
  const [menuPosition, setMenuPosition] = useState({ x: 0, y: 0 });
  const [menuFsPath, setMenuFsPath] = useState("");
  const [menuOnDir, setMenuOnDir] = useState(false);
  const [curFileName, setCurFileName] = useState("");
  const [curFileContent, setCurFileContent] = useState(defaultJsCode);
  const [isNewFileDialogOpen, setIsNewFileDialogOpen] = useState(false);

  const handleContextMenu = (
    event: React.MouseEvent<Element, MouseEvent>,
    isDir: boolean,
    fsPath: string
  ) => {
    event.preventDefault();
    setMenuOpen(true);
    setMenuPosition({ x: event.clientX, y: event.clientY });
    setMenuFsPath(fsPath);
    setMenuOnDir(isDir);
  };

  const handleNewFileMenu = () => {
    setIsNewFileDialogOpen(true);
  };

  const handleNewFileName = (name: string) => {
    setCurFileName(name);
  };

  useEffect(() => {
    const listCodeUrl = "http://localhost:3457/list_code/8nvcym53y8d2";
    const deployCodeUrl = "http://localhost:3457/deploy_code/8nvcym53y8d2";
    axios
      .get(listCodeUrl, {
        timeout: 4000,
      })
      .then((response) => {
        // hard code api.js right now.
        console.log("list_code returns");
        console.log(response.data);
        const { codes } = response.data as {
          codes: { fs_path: string; content: string }[];
        };

        if (codes.length > 0) {
          setCurFileName(codes[0]!.fs_path);
          setCurFileContent(codes[0]!.content);
        }
        setIsCodeLoading(false);
      })
      .catch((error) => console.error("list_code error: ", error));
  }, []);

  const prevFileContent = useRef(defaultJsCode);
  useEffect(() => {
    const interval = setInterval(() => {
      const deployCodeUrl = "http://localhost:3457/deploy_code/8nvcym53y8d2";

      if (curFileContent !== prevFileContent.current) {
        console.log("deploy_code: ", curFileContent);
        axios
          .post(
            deployCodeUrl,
            {
              codes: [{ fs_path: "functions/api.js", content: curFileContent }],
            },
            { timeout: 2000, headers: { "Content-Type": "application/json" } }
          )
          .then((response) => {
            console.log("deploy_code returns");
          })
          .catch((error) => console.error("deploy_code error: ", error));
      }
      prevFileContent.current = curFileContent;
    }, 3000);

    return () => clearInterval(interval);
  }, [curFileName, curFileContent]);

  return (
    <div>
      {isCodeLoading ? (
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
      ) : (
        <div>
          <div>
            {isMenuOpen && (
              <FileMenu
                menuPosition={menuPosition}
                hideMenu={() => setMenuOpen(false)}
                handleNewFile={handleNewFileMenu}
              ></FileMenu>
            )}
          </div>
          <div>
            {isNewFileDialogOpen && (
              <NewFileDialog
                closeDialog={() => setIsNewFileDialogOpen(false)}
                handleNewFileName={handleNewFileName}
              ></NewFileDialog>
            )}
          </div>
          <div className="grid grid-cols-12 gap-2">
            <div className="col-span-2 rounded border-gray-300 bg-white p-2 shadow-sm ring-4 ring-sky-500/0 transition-colors">
              <TreeView
                data={data}
                aria-label="directory tree"
                nodeRenderer={({
                  element,
                  isBranch,
                  isExpanded,
                  getNodeProps,
                  level,
                }) => {
                  const isDir = isBranch;
                  const fsPath = element.metadata?.fsPath as string;

                  return (
                    <div
                      {...getNodeProps()}
                      style={{ paddingLeft: 20 * (level - 1) }}
                      onContextMenu={(event) => {
                        handleContextMenu(event, isDir, fsPath);
                      }}
                    >
                      {isBranch ? <FolderIcon isOpen={isExpanded} /> : null}
                      {element.name}
                    </div>
                  );
                }}
              />
            </div>
            <div className="col-span-10 cursor-default rounded p-2 shadow-sm ring-4 ring-sky-500/0 transition-colors">
              {curFileName != "" && (
                <FileContent
                  name={curFileName}
                  content={curFileContent}
                  handleCodeChange={(code: string) => {
                    setCurFileContent(code);
                  }}
                ></FileContent>
              )}
            </div>
          </div>
        </div>
      )}
    </div>
  );
}

const FolderIcon = ({ isOpen }: { isOpen: boolean }) =>
  isOpen ? (
    <FaRegFolderOpen color="e8a87c" className="inline-block" />
  ) : (
    <FaRegFolder color="e8a87c" className="inline-block" />
  );

export default DirectoryTreeView;
