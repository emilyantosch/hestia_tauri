import { SidebarFolder, Folder } from "../molecules/sidebarFolder";
import { Sidebar, SidebarContent } from "../ui/sidebar";

let folder: Folder = {
  title: "Vault",
  subfolder: [
    {
      title: "Folder 1",
      subfolder: [
        {
          title: "Subfolder 1",
          subfolder: [
            {
              title: "Subsubfolder 1",
              subfolder: new Array<Folder>
            }
          ]
        }
      ]
    },
    {
      title: "Folder 2",
      subfolder: new Array<Folder>
    },
  ]
}


export function AppSidebar() {
  return (
    <Sidebar>
      <SidebarContent>
        <SidebarFolder title={folder.title} subfolder={folder.subfolder} ></SidebarFolder>
      </SidebarContent>
    </Sidebar>
  );
}
