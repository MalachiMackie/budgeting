import { Group, ScrollArea, Text } from "@mantine/core";
import { FC } from "react";
import { LinksGroup } from "./LinksGroup/LinksGroup";
import { NavLink, NavLinkProps } from "./NavLink";
import "./SideNav.css";

// From here: https://ui.mantine.dev/component/navbar-nested/

type Group = {
  label: string;
  icon: FC<any>;
  links: Omit<NavLinkProps, "isRoot">[];
};

export type SideNavProps = {
  items: (
    | ({ type: "group" } & Group)
    | ({ type: "link" } & Omit<NavLinkProps, "isRoot">)
  )[];
};

export function SideNav({ items: groups }: SideNavProps): JSX.Element {
  const links = groups.map((x) => {
    switch (x.type) {
      case "group":
        return <LinksGroup key={x.label} {...x} />;
      case "link":
        return <NavLink isRoot key={x.label} {...x} />;
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
