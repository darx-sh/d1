import { useState } from "react";
import { useEffectOnce } from "usehooks-ts";
import TableList from "~/components/project/TableList";
import TableDetails from "~/components/project/TableDetails";
import { useProjectState } from "~/components/project/ProjectContext";
import { env } from "~/env.mjs";
import axios from "axios";

export default function Database() {
  const [isLoading, setIsLoading] = useState(true);
  const projectState = useProjectState()!;
  const envId = projectState.envInfo!.id;

  useEffectOnce(() => {
    const listTableUrl = `${env.NEXT_PUBLIC_DATA_PLANE_URL}/invoke/_plugins/schema/api.listTable`;
    axios
      .post(
        listTableUrl,
        {},
        {
          headers: { "Darx-Dev-Host": `${envId}.darx.sh` },
        }
      )
      .then((response) => {
        console.log(response.data);
      })
      .catch((error) => console.log("invoke function error: ", error));
  });

  return (
    <div className=" flex h-full border-2 pt-2">
      <div className="w-40 bg-white">
        <TableList></TableList>
      </div>
      <div className="ml-2 flex-1 bg-white">
        <TableDetails></TableDetails>
      </div>
    </div>
  );
}
