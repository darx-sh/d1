import React, { useState } from "react";
import { FaRegFolder, FaRegFolderOpen } from "react-icons/fa";
import TreeView, { flattenTree } from "react-accessible-treeview";
import FileMenu from "~/components/project/FileMenu";

// const folder = {
//   name: "",
//   children: [
//     {
//       name: "functions",
//       children: [{ name: "foo.js" }, { name: "bar.ts" }, { name: "foo_dir" }],
//     },
//     {
//       name: "lib",
//       children: [
//         {
//           name: "a.js",
//         },
//         { name: "b.js" },
//       ],
//     },
//     { name: "foo_dir", children: [{ name: "aa.js" }] },
//   ],
// };

// const data = flattenTree(folder);

const data = [
  { id: "root", name: "", children: ["functions", "lib"], parent: null },
  {
    id: "functions",
    name: "functions",
    children: [],
    parent: "root",
    isBranch: true,
  },
  {
    id: "lib",
    name: "lib",
    children: [],
    parent: "root",
    isBranch: true,
  },
];

function DirectoryTreeView() {
  const [isMenuOpen, setMenuOpen] = useState(false);
  const [menuPosition, setMenuPosition] = useState({ x: 0, y: 0 });

  return (
    <div>
      <div className="bg-slate-200 p-2">
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
            const handleContextMenu = (
              event: React.MouseEvent<Element, MouseEvent>
            ) => {
              event.preventDefault();
              setMenuOpen(true);
              setMenuPosition({ x: event.clientX, y: event.clientY });
              console.log("context menu ", event.clientX, event.clientY);
            };

            const hideMenu = () => {
              setMenuOpen(false);
            };
            return (
              <div
                {...getNodeProps()}
                style={{ paddingLeft: 20 * (level - 1) }}
                onContextMenu={handleContextMenu}
              >
                {isMenuOpen && (
                  <FileMenu
                    menuPosition={menuPosition}
                    hideMenu={hideMenu}
                  ></FileMenu>
                )}
                {isBranch ? <FolderIcon isOpen={isExpanded} /> : null}
                {element.name}
              </div>
            );
          }}
        />
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
