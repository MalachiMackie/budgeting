import { Group, ScrollArea, Text } from "@mantine/core";
import { Icon2fa } from "@tabler/icons-react";
import { LinksGroup } from "./LinksGroup/LinksGroup";
import "./SideNav.css";

// From here: https://ui.mantine.dev/component/navbar-nested/

export function SideNav(): JSX.Element {
  const links: JSX.Element[] = [
    <LinksGroup
      label="Something"
      icon={Icon2fa}
      links={[{ label: "somewhere", link: "somewhere" }]}
    />,
  ];

  return (
    <nav className={"navbar"}>
      <div className={"header"}>
        <Group justify="space-between">
          <Text>Budgeting</Text>
        </Group>
      </div>

      <ScrollArea className={"links"}>
        <div className={"linksInner"}>{links}</div>
      </ScrollArea>

      <div className={"footer"}>{/* <UserButton /> */}</div>
    </nav>
  );
}
