import { createContext, useContext, ReactNode, Dispatch } from "react";
import { useImmerReducer } from "use-immer";
import { INode } from "react-accessible-treeview";

type ProjectState = {
  directory: {
    codes: { fsPath: string; content: string }[];
    httpRoutes: { jsEntryPoint: string; jsExport: string; httpPath: string }[];
    curOpenCodeIdx: number | null;
    treeViewData: INode[];
  };

  tabs: { tabType: TabType; meta: any }[];
  curOpenTabIdx: number | null;
};

const initialProject: ProjectState = {
  directory: {
    codes: [],
    httpRoutes: [],
    curOpenCodeIdx: null,
    treeViewData: [],
  },
  tabs: [],
  curOpenTabIdx: null,
};

enum TabType {
  JsEditor,
  Database,
}

type ProjectAction =
  | {
      type: "LoadCodes";
      codes: { fsPath: string; content: string }[];
      httpRoutes: {
        jsEntryPoint: string;
        jsExport: string;
        httpPath: string;
      }[];
    }
  | { type: "NewJsFile"; fsPath: string }
  | { type: "UpdateJsFile"; fsPath: string; content: string }
  | { type: "RenameJsFile"; oldFsPath: string; newFsPath: string }
  | { type: "RenameDirectory"; oldFsPath: string; newFsPath: string }
  | { type: "DeleteJsFile"; fsPath: string }
  | { type: "DeleteDirectory"; fsPath: string }
  | { type: "DoubleClickJsFile"; fsPath: string }
  | { type: "CloseJsTab"; fsPath: string }
  | { type: "SelectTab"; tabIdx: number }
  | {
      type: "UpdateHttpRoutes";
      httpRoutes: {
        jsEntryPoint: string;
        jsExport: string;
        httpPath: string;
      }[];
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

const initialEditorCode = `\
export default function foo() {
  return "hello from darx";
}
`;

function projectReducer(
  state: ProjectState,
  action: ProjectAction
): ProjectState {
  switch (action.type) {
    case "LoadCodes": {
      state.directory.codes = action.codes;
      state.directory.httpRoutes = action.httpRoutes;
      state.directory.curOpenCodeIdx = null;
      state.directory.treeViewData = buildTreeViewData(state.directory.codes);
      return state;
    }
    case "NewJsFile": {
      state.directory.codes.push({
        fsPath: action.fsPath,
        content: initialEditorCode,
      });
      state.directory.treeViewData = buildTreeViewData(state.directory.codes);
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
