// The code of react-tree-view is copied from https://github.com/dgreene1/react-accessible-treeview
// with some fixes on TypeScript types.
import TreeView, {
  ITreeViewOnExpandProps,
  ITreeViewOnSelectProps,
  ITreeViewOnNodeSelectProps,
  ITreeViewProps,
  ITreeViewOnLoadDataProps,
} from "./TreeView";
import { CLICK_ACTIONS } from "./TreeView/constants";
import { ITreeViewState, TreeViewAction } from "./TreeView/reducer";
import {
  INode,
  NodeId,
  EventCallback,
  ClickActions,
  INodeRendererProps,
  IBranchProps,
  LeafProps,
} from "./TreeView/types";
import { flattenTree } from "./TreeView/utils";

export type {
  flattenTree,
  ITreeViewProps,
  INode,
  NodeId,
  ITreeViewOnSelectProps,
  ITreeViewOnNodeSelectProps,
  CLICK_ACTIONS,
  ITreeViewOnExpandProps,
  ITreeViewOnLoadDataProps,
  EventCallback,
  TreeViewAction,
  INodeRendererProps,
  ClickActions,
  IBranchProps,
  LeafProps,
  ITreeViewState,
};
export default TreeView;
