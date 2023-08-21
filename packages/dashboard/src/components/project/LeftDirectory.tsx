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
} from "~/components/project/ProjectContext";
import { ITreeViewOnNodeSelectProps } from "~/components/react-tree-view";
import { classNames } from "~/utils";

type MenuPosition = {
  coord: { x: number; y: number } | null;
  nodeParent: NodeId | null;
  nodeIsBranch: boolean | undefined;
  currentNodeId: NodeId;
};

export default function LeftDirectory() {
  const [expandedIds, setExpandedIds] = useState<NodeId[]>(["/functions"]);
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

  const handleNodeSelect = (nodeSelectProps: ITreeViewOnNodeSelectProps) => {
    const { element } = nodeSelectProps;
    if (!element.isBranch) {
      projectDispatch!({
        type: "OpenJsFile",
        nodeId: element.id,
      });
    }
  };

  return (
    <>
      <div>
        {menuPosition.coord && (
          <FileMenu
            menuPosition={menuPosition.coord}
            hideMenu={() => setMenuPosition({ ...menuPosition, coord: null })}
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
            <div
              className={classNames(
                isSelected && !isBranch ? "bg-blue-100" : "",
                "mt-4 pl-3"
              )}
            >
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
  );
}
