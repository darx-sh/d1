import { useProjectState } from "./ProjectContext";
import Database from "~/components/project/database/Database";
import Playground from "~/components/project/playground/Playground";
import React from "react";

export default function RightContainer() {
  const state = useProjectState();

  switch (state.curNav) {
    case "playground":
      return <Playground></Playground>;
    case "functions": {
      return <div>API List</div>;
    }
    case "database": {
      return <Database></Database>;
    }
    default:
      throw new Error("not implemented");
  }
}
