import { createContext, useContext, ReactNode, Dispatch } from "react";
import { useImmerReducer } from "use-immer";
import { INode, NodeId } from "~/components/react-tree-view";
import md5 from "crypto-js/md5";

type ProjectState = {
  directory: {
    codes: {
      fsPath: string;
      content: string;
      prevChecksum?: string;
      curChecksum?: string;
    }[];
    httpRoutes: HttpRoute[];
    treeViewData: INode[];
  };

  tabs: Tab[];
  curOpenTabIdx: number | null;
  projectInfo: ProjectInfo | null;
  envInfo: EnvInfo | null;
};

export type ProjectInfo = {
  id: string;
  name: string;
};

export type EnvInfo = {
  id: string;
  name: string;
};

export type HttpRoute = {
  jsEntryPoint: string;
  jsExport: string;
  httpPath: string;
  method: string;
  curParams: string;
};

type Tab = { type: "JsEditor"; codeIdx: number } | { type: "Database" };

export const initialHttpParam = JSON.stringify({}, null, 2);

const initialProject: ProjectState = {
  directory: {
    codes: [],
    httpRoutes: [],
    treeViewData: [],
  },
  tabs: [],
  curOpenTabIdx: null,
  projectInfo: null,
  envInfo: null,
};

enum TabType {
  JsEditor,
  Database,
}

type ProjectAction =
  | { type: "SetProject"; project: ProjectInfo }
  | { type: "SetCurEnv"; envId: string }
  | {
      type: "LoadEnv";
      codes: { fsPath: string; content: string }[];
      httpRoutes: HttpRoute[];
      projectInfo: ProjectInfo;
      envInfo: EnvInfo;
    }
  | { type: "PersistedCode"; checksums: CodeChecksums; httpRoutes: HttpRoute[] }
  | { type: "NewJsFile"; parentNodeId: NodeId; fileName: string }
  | { type: "OpenJsFile"; nodeId: NodeId }
  | { type: "UpdateJsFile"; codeIdx: number; content: string }
  | { type: "RenameJsFile"; oldFsPath: string; newFsPath: string }
  | { type: "RenameDirectory"; oldFsPath: string; newFsPath: string }
  | { type: "DeleteJsFile"; fsPath: string }
  | { type: "DeleteDirectory"; fsPath: string }
  | { type: "DoubleClickJsFile"; fsPath: string }
  | { type: "CloseJsTab"; fsPath: string }
  | { type: "SelectTab"; tabIdx: number }
  | { type: "UpdatePostParam"; httpRoute: HttpRoute; param: string }
  | { type: "OpenDatabase" };

export type CodeChecksums = {
  [key: string]: string;
};

const ProjectStateContext = createContext<ProjectState | null>(null);
const ProjectDispatchContext = createContext<Dispatch<ProjectAction> | null>(
  null
);

export function ProjectProvider({ children }: { children: ReactNode }) {
  const [projectState, projectDispatch] = useImmerReducer(
    projectReducer,
    initialProject
  );

  return (
    <ProjectStateContext.Provider value={projectState}>
      <ProjectDispatchContext.Provider value={projectDispatch}>
        {children}
      </ProjectDispatchContext.Provider>
    </ProjectStateContext.Provider>
  );
}

export function useProjectState() {
  return useContext(ProjectStateContext);
}

export function useProjectDispatch() {
  return useContext(ProjectDispatchContext);
}

function initialEditorCode(fsPath: string) {
  return `\
export default function foo() {
  return "hello from ${fsPath}";
}
  `;
}

function projectReducer(
  state: ProjectState,
  action: ProjectAction
): ProjectState {
  switch (action.type) {
    case "LoadEnv": {
      const codes = action.codes.map((c) => {
        const digest = md5(c.content).toString();
        return {
          ...c,
          prevChecksum: digest,
          curChecksum: digest,
        };
      });
      state.projectInfo = action.projectInfo;
      state.envInfo = action.envInfo;
      state.directory.codes = codes;
      state.directory.httpRoutes = action.httpRoutes;
      state.directory.treeViewData = buildTreeViewData(state.directory.codes);
      return state;
    }
    case "PersistedCode": {
      state.directory.codes = state.directory.codes.map((c) => {
        const checksum = action.checksums[c.fsPath];
        if (checksum) {
          return { ...c, prevChecksum: checksum };
        } else {
          return c;
        }
      });
      state.directory.httpRoutes = action.httpRoutes;
      return state;
    }
    case "NewJsFile": {
      const fsPath = newFsPath(action.parentNodeId, action.fileName);
      const content = initialEditorCode(fsPath);
      state.directory.codes.push({
        fsPath,
        content: content,
        curChecksum: md5(content).toString(),
      });
      state.directory.treeViewData = buildTreeViewData(state.directory.codes);
      // create a new tab
      state.tabs.push({
        type: "JsEditor",
        codeIdx: state.directory.codes.length - 1,
      });
      state.curOpenTabIdx = state.tabs.length - 1;
      return state;
    }
    case "OpenJsFile": {
      const fsPath = nodeIdToFsPath(action.nodeId);
      const codeIdx = state.directory.codes.findIndex(
        (c) => c.fsPath === fsPath
      );
      if (codeIdx < 0) {
        throw new Error(
          `Cannot find code with fsPath: ${fsPath}, nodeId: ${action.nodeId}`
        );
      }
      // state.curOpenTabIdx = codeIdx;
      const tabIdx = state.tabs.findIndex(
        (t) => t.type === "JsEditor" && t.codeIdx === codeIdx
      );
      if (tabIdx >= 0) {
        // we already find the tab, just select it.
        state.curOpenTabIdx = tabIdx;
      } else {
        // create a new tab.
        state.tabs.push({ type: "JsEditor", codeIdx: codeIdx });
        state.curOpenTabIdx = state.tabs.length - 1;
      }
      return state;
    }
    case "UpdateJsFile": {
      state.directory.codes[action.codeIdx]!.content = action.content;
      state.directory.codes[action.codeIdx]!.curChecksum = md5(
        action.content
      ).toString();
      return state;
    }
    case "SelectTab": {
      state.curOpenTabIdx = action.tabIdx;
      return state;
    }
    case "UpdatePostParam": {
      const { httpRoute, param } = action;
      const idx = state.directory.httpRoutes.findIndex((r) => {
        return (
          r.httpPath === httpRoute.httpPath && r.method === httpRoute.method
        );
      });
      if (idx >= 0) {
        state.directory.httpRoutes[idx]!.curParams = param;
      }
      return state;
    }
    case "OpenDatabase": {
      const tabIdx = state.tabs.findIndex((t) => t.type === "Database");
      if (tabIdx >= 0) {
        // we already find the tab, just select it.
        state.curOpenTabIdx = tabIdx;
      } else {
        state.tabs.push({ type: "Database" });
        state.curOpenTabIdx = state.tabs.length - 1;
      }
      return state;
    }
    default:
      throw "Unhandled action type: " + action.type + " in projectReducer";
  }
}

// [
//  {fsPath: "functions/foo.js", content: ""},
//  {fsPath: "functions/bar.js", content: ""},
//  {fsPath: "functions/foo/foo.js", content: ""]
//
// [
//  {id: "", name: "", parent: null, children: ["functions"]},
//  {id: "functions", name: "functions", parent: "", children: ["foo.js", "bar.js", "foo"], isBranch: true},
//  {id: "functions/foo", name: "foo", parent: "functions", children: ["functions/foo/foo.js"], isBranch: true},
//  {id: "functions/foo/foo.js", name: "foo.js", parent: "functions/foo", children: [], isBranch: false},
//  {id: "functions/foo.js", name: "foo.js", parent: "functions", children: [], isBranch: false},
//  {id: "functions/bar.js", name: "bar.js", parent: "functions", children: [], isBranch: false},
// ]
function buildTreeViewData(
  codes: { fsPath: string; content: string }[]
): INode[] {
  const rootNode: INode = {
    id: "/",
    name: "",
    parent: null,
    children: [],
  };

  const nodeList = [rootNode];

  function getFileName(filePath: string): string {
    return filePath.split("/").pop() || "";
  }

  // build all directory nodes
  for (const code of codes) {
    const filePath = code.fsPath;
    if (filePath.lastIndexOf("/") > 0) {
      const dirPath = filePath.slice(0, filePath.lastIndexOf("/"));
      const dirNames = dirPath.split("/");

      let currentNode = rootNode;
      let currentDirPath = "";
      for (const dirName of dirNames) {
        currentDirPath += "/" + dirName;

        const existingDirNode = nodeList.find(
          (node) => node.id === currentDirPath
        );

        if (!existingDirNode) {
          const newDirNode: INode = {
            id: currentDirPath,
            name: dirName,
            parent: currentNode.id,
            children: [],
            isBranch: true,
          };
          currentNode = newDirNode;
          nodeList.push(currentNode);
        } else {
          currentNode = existingDirNode;
        }
      }
      // add current file
      const fileName = getFileName(filePath);
      const newFileNode: INode = {
        id: currentDirPath + "/" + fileName,
        name: fileName,
        parent: currentNode.id,
        children: [],
        isBranch: false,
      };
      nodeList.push(newFileNode);
    } else {
      // file in root directory
      const fileName = getFileName(filePath);
      const newFileNode: INode = {
        id: "/" + fileName,
        name: fileName,
        parent: rootNode.id,
        children: [],
        isBranch: false,
      };
      nodeList.push(newFileNode);
    }
  }

  if (codes.length === 0) {
    nodeList.push({
      id: "/functions",
      name: "functions",
      parent: "/",
      children: [],
      isBranch: true,
    });
  }

  // build parent-child relationship between directory nodes,
  // since the "parent" field is already set, we only need
  // to set the "children" field.
  for (const node of nodeList) {
    if (node.parent) {
      const parentNode = nodeList.find((n) => n.id === node.parent);
      if (parentNode) {
        parentNode.children.push(node.id);
      } else {
        throw new Error("parent node not found");
      }
    }
  }
  return nodeList;
}

function nodeIdToFsPath(nodeId: NodeId): string {
  const id = nodeId as string;
  if (id.startsWith("/")) {
    return id.replace("/", "");
  } else {
    throw new Error(`invalid nodeId: ${nodeId}`);
  }
}

function newFsPath(parentNodeId: NodeId, fileName: string): string {
  const parentId = parentNodeId as string;
  if (parentId.startsWith("/")) {
    return parentId.replace("/", "") + "/" + fileName;
  } else {
    throw new Error(`invalid parentNodeId: ${parentNodeId}`);
  }
}
