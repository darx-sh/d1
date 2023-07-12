import React from "react";
import { FaList, FaRegFolder, FaRegFolderOpen } from "react-icons/fa";
import TreeView, { flattenTree } from "react-accessible-treeview";
import { SiJavascript } from "react-icons/si";
import { SiTypescript } from "react-icons/si";

const folder = {
  name: "",
  children: [
    {
      name: "functions",
      children: [{ name: "foo.js" }, { name: "bar.ts" }],
    },
    {
      name: "lib",
      children: [
        {
          name: "a.js",
        },
        { name: "b.js" },
      ],
    },
  ],
};

const data = flattenTree(folder);

function DirectoryTreeView() {
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
          }) => (
            <div {...getNodeProps()} style={{ paddingLeft: 20 * (level - 1) }}>
              {isBranch ? <FolderIcon isOpen={isExpanded} /> : null}
              {element.name}
            </div>
          )}
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
