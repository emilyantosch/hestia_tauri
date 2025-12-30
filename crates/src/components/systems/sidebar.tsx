import { SidebarFolder, Folder } from "../molecules/sidebarFolder";
import { Sidebar, SidebarContent } from "../ui/sidebar";

let folder: Folder = {
  title: "Vault",
  subfolder: [
    {
      title: "Folder 1",
      subfolder: [
        {
          title: "Subfolder-1",
          subfolder: [
            {
              title: "Subsubfolder-1",
              subfolder: [
                {
                  title: "SubSubsubfolder 1",
                  subfolder: new Array<Folder>
                },
                {
                  title: "SubSubsubfolder 2",
                  subfolder: [
                    {
                      title: "SubSubSubsubfolder 1",
                      subfolder: new Array<Folder>
                    },
                    {
                      title: "SubSubSubsubfolder 2",
                      subfolder: new Array<Folder>
                    }
                  ]
                }
              ]
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
        <div className="flex-col justify-between items-start">
          <SidebarFolder title={folder.title} subfolder={folder.subfolder} ></SidebarFolder>
        </div>
      </SidebarContent>
    </Sidebar>
  );
}
