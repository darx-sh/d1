import { useProjectState } from "./ProjectContext";
import Database from "~/components/project/database/Database";
import Functions from "~/components/project/functions/Functions";
import React from "react";

export default function RightContainer() {
  const state = useProjectState();

  switch (state.curNav) {
    case "functions": {
      return <Functions></Functions>;
    }
    case "database": {
      return <Database></Database>;
    }
    default:
      throw new Error("not implemented");
  }
}
