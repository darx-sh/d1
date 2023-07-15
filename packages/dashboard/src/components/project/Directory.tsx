import React, { useState } from "react";
import { FaRegFolder, FaRegFolderOpen } from "react-icons/fa";
import TreeView, { flattenTree } from "react-accessible-treeview";
import FileMenu from "~/components/project/FileMenu";
import FileContent from "~/components/project/FileContent";
import NewFileDialog from "~/components/project/NewFileDialog";

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

  const hideMenu = () => {
    setMenuOpen(false);
  };

  const handleNewFileMenu = () => {
    setIsNewFileDialogOpen(true);
  };

  const handleNewFileName = (name: string) => {
    console.log("new file name: ", name, "fsPath: ", menuFsPath);
    setCurFileName(name);
  };

  const closeDialog = () => {
    setIsNewFileDialogOpen(false);
  };

  return (
    <div>
      <div>
        {isMenuOpen && (
          <FileMenu
            menuPosition={menuPosition}
            hideMenu={hideMenu}
            handleNewFile={handleNewFileMenu}
          ></FileMenu>
        )}
      </div>
      <div>
        {isNewFileDialogOpen && (
          <NewFileDialog
            closeDialog={closeDialog}
            handleNewFileName={handleNewFileName}
          ></NewFileDialog>
        )}
      </div>
      <div className="grid grid-cols-12 gap-2 ">
        <div className="col-span-2 bg-slate-50 p-2">
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
        <div className="col-span-10">
          {curFileName != "" && (
            <FileContent
              name={curFileName}
              content={curFileContent}
            ></FileContent>
          )}
        </div>
      </div>
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
