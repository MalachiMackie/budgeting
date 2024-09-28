import { Group, ScrollArea, Text } from "@mantine/core";
import { LinksGroup, LinksGroupProps } from "./LinksGroup/LinksGroup";
import { NavButton, NavButtonProps } from "./NavButton";
import { NavLink, NavLinkProps } from "./NavLink";
import "./SideNav.css";

// From here: https://ui.mantine.dev/component/navbar-nested/

export type SideNavProps = {
  items: (
    | ({ type: "group" } & LinksGroupProps)
    | ({ type: "link" } & NavLinkProps)
    | ({ type: "button" } & NavButtonProps)
  )[];
};

export function SideNav({ items: groups }: SideNavProps): JSX.Element {
  const links = groups.map((x) => {
    switch (x.type) {
      case "group":
        return <LinksGroup key={x.label} {...x} />;
      case "link":
        return <NavLink isRoot key={x.label} {...x} />;
      case "button":
        return <NavButton isRoot key={x.label} {...x} />;
    }
  });

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
