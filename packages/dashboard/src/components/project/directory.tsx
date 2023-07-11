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
              {isBranch ? (
                <FolderIcon isOpen={isExpanded} />
              ) : (
                <FileIcon filename={element.name} />
              )}

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
    <FaRegFolderOpen color="e8a87c" className="icon" />
  ) : (
    <FaRegFolder color="e8a87c" className="icon" />
  );

const FileIcon = ({ filename }: { filename: string }) => {
  const extension = filename.slice(filename.lastIndexOf(".") + 1);
  switch (extension) {
    case "js":
      return <SiJavascript className="icon" />;
    case "ts":
      return <SiTypescript className="icon" />;
    default:
      return null;
  }
};

export default DirectoryTreeView;
