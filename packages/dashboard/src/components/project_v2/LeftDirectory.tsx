import React, { useEffect, useRef, useState } from "react";
import FileMenu from "~/components/project/FileMenu";
import NewFileDialog from "~/components/project/NewFileDialog";
import TreeView, { flattenTree } from "react-accessible-treeview";
import { FolderIcon, FolderOpenIcon } from "@heroicons/react/24/solid";

const folder = {
  name: "",
  children: [
    {
      name: "functions",
      children: [{ name: "foo.js" }, { name: "bar.js" }],
    },
  ],
};

const data = flattenTree(folder);

export default () => {
  const [isMenuOpen, setMenuOpen] = useState(false);
  const [menuPosition, setMenuPosition] = useState({ x: 0, y: 0 });
  const [menuFsPath, setMenuFsPath] = useState("");
  const [menuOnDir, setMenuOnDir] = useState(false);

  const handleContextMenu = (
    event: React.MouseEvent<Element, MouseEvent>,
    isDir: boolean,
    fsPath: string
  ) => {
    // event.preventDefault();
    // setMenuOpen(true);
    // setMenuPosition({ x: event.clientX, y: event.clientY });
    // setMenuFsPath(fsPath);
    // setMenuOnDir(isDir);
  };

  return (
    <div>
      <div>
        {isMenuOpen && (
          <FileMenu
            menuPosition={menuPosition}
            hideMenu={() => setMenuOpen(false)}
            handleNewFile={() => console.log("new file")}
          ></FileMenu>
        )}
      </div>
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
            <div className="pl-3">
              <div
                {...getNodeProps()}
                style={{ paddingLeft: 20 * (level - 1) }}
                onContextMenu={(event) => {
                  handleContextMenu(event, isDir, fsPath);
                }}
              >
                {isBranch ? <FolderIcon className="mr-1 h-4 w-4" /> : null}
                <span className="-mb-0.5">{element.name}</span>
              </div>
            </div>
          );
        }}
      />
    </div>
  );
};
