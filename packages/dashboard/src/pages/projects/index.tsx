import Link from "next/link";
import { useRouter } from "next/router";
import { useState } from "react";
import { useEffectOnce } from "usehooks-ts";
import axios from "axios";
import axiosRetry from "axios-retry";
import Spinner from "~/components/project/Spinner";
import {
  ProjectInfo,
  ProjectProvider,
  EnvInfo,
} from "~/components/project/ProjectContext";
import randomName from "~/components/project/RandomName";
import TopNav from "~/components/TopNav";

const user = {
  id: "Tom Cook",
  email: "tom@example.com",
  imageUrl:
    "https://images.unsplash.com/photo-1472099645785-5658abf4ff4e?ixlib=rb-1.2.1&ixid=eyJhcHBfaWQiOjEyMDd9&auto=format&fit=facearea&facepad=2&w=256&h=256&q=80",
};
const navigation = [{ id: "Projects", href: "#", current: true }];
const userNavigation = [{ id: "Sign out", href: "#" }];

const orgId = "org_123";

function classNames(...classes: any[]) {
  return classes.filter(Boolean).join(" ");
}

type ListProjectRsp = {
  projects: ProjectInfo[];
};

type NewProjectRsp = {
  project: ProjectInfo;
  env: EnvInfo;
};

function Projects() {
  const [isLoading, setIsLoading] = useState(true);
  const [projects, setProjects] = useState<ProjectInfo[]>([]);
  const router = useRouter();

  const navToProject = (project: ProjectInfo) => {
    // eslint-disable-next-line
    router.push(`/projects/${project.id}`);
  };

  const handleNewProject = () => {
    const url = "http://localhost:3457/new_tenant_project";
    axios
      .post(url, { org_id: orgId, project_name: randomName() })
      .then((response) => {
        const { project } = response.data as NewProjectRsp;
        navToProject(project);
      })
      .catch((err) => {
        console.error("failed to create project");
      });
  };

  useEffectOnce(() => {
    const listProjectUrl = `http://localhost:3457/list_project/${orgId}`;
    const instance = axios.create();
    axiosRetry(instance, {
      retries: 100,
      shouldResetTimeout: true,
      retryDelay: (retryCount) => {
        return 1000;
      },
    });
    instance
      .get(listProjectUrl, { timeout: 4000 })
      .then((response) => {
        const { projects } = response.data as ListProjectRsp;
        setIsLoading(false);
        setProjects(projects);
      })
      .catch((err) => {
        console.error(err);
      });
  });

  return (
    <>
      {isLoading ? (
        <Spinner />
      ) : (
        <div className="min-h-full">
          <TopNav navs={[{ name: "Home", href: "/" }]}></TopNav>

          <div className="my-10 px-60">
            <button
              onClick={handleNewProject}
              type="button"
              className="mb-12 rounded-md bg-indigo-600 px-3 py-2 text-sm font-semibold text-white shadow-sm hover:bg-indigo-500 focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-indigo-600"
            >
              New Project
            </button>
            <ul role="list" className="grid grid-cols-2 gap-4">
              {projects.map((project) => (
                <li
                  key={project.id}
                  className="rounded-lg ring-1 ring-inset ring-gray-300 transition-colors duration-200 hover:bg-gray-100"
                >
                  <Link
                    onClick={(e) => {
                      e.preventDefault();
                      navToProject(project);
                    }}
                    href={`/projects/${project.id}}`}
                    className="flex place-content-center py-5"
                  >
                    <h2>
                      <strong className="align-middle text-base font-medium leading-tight">
                        {project.name}
                      </strong>
                    </h2>
                  </Link>
                </li>
              ))}
            </ul>
          </div>
        </div>
      )}
    </>
  );
}

export default function ProjectsWrapper() {
  return (
    <ProjectProvider>
      <Projects></Projects>
    </ProjectProvider>
  );
}
