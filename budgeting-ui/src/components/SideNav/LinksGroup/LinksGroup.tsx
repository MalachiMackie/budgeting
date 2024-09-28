import { Collapse } from "@mantine/core";
import { useState } from "react";
import { NavButton, NavButtonProps } from "../NavButton";
import { NavLink, NavLinkProps } from "../NavLink";
import { SideNavItem } from "../SideNavItem";
import "./LinksGroup.css";

export type LinksGroupProps = {
  icon: React.FC<any>;
  label: string;
  initiallyOpened?: boolean;
  links: LinkOrButton[];
};

type LinkOrButton =
  | ({ type: "link" } & NavLinkProps)
  | ({ type: "button" } & NavButtonProps);

export function LinksGroup({
  icon,
  label,
  initiallyOpened,
  links,
}: LinksGroupProps) {
  const hasLinks = Array.isArray(links);
  const [opened, setOpened] = useState(initiallyOpened || false);

  const items = (hasLinks ? links : []).map((link) => {
    switch (link.type) {
      case "link":
        return <NavLink key={link.id} {...link} />;
      case "button":
        return <NavButton key={link.id} {...link} />;
    }
  });

  return (
    <>
      <SideNavItem
        className={"control"}
        label={label}
        icon={icon}
        openable={hasLinks}
        open={opened}
        onOpenToggled={() => setOpened((o) => !o)}
      />
      {hasLinks ? <Collapse in={opened}>{items}</Collapse> : null}
    </>
  );
}
