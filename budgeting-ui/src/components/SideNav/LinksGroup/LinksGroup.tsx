import { Box, Collapse } from "@mantine/core";
import { IconCalendarStats } from "@tabler/icons-react";
import { useState } from "react";
import { NavLink, NavLinkProps } from "../NavLink";
import { SideNavItem } from "../SideNavItem";
import "./LinksGroup.css";

interface LinksGroupProps {
  icon: React.FC<any>;
  label: string;
  initiallyOpened?: boolean;
  links?: NavLinkProps[];
}

export function LinksGroup({
  icon,
  label,
  initiallyOpened,
  links,
}: LinksGroupProps) {
  const hasLinks = Array.isArray(links);
  const [opened, setOpened] = useState(initiallyOpened || false);

  const items = (hasLinks ? links : []).map((link) => (
    <NavLink key={link.label} {...link} />
  ));

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

const mockdata = {
  label: "Releases",
  icon: IconCalendarStats,
  links: [
    { label: "Upcoming releases", link: "/" },
    { label: "Previous releases", link: "/" },
    { label: "Releases schedule", link: "/" },
  ],
};

export function NavbarLinksGroup() {
  return (
    <Box mih={220} p="md">
      <LinksGroup {...mockdata} />
    </Box>
  );
}
