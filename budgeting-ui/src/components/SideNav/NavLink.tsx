import { Flex, rem, Text, ThemeIcon } from "@mantine/core";
import { FC } from "react";
import { Link } from "react-router-dom";

export type NavLinkProps = {
  link: string;
  label: string;
  subLabel?: string;
  isRoot?: boolean;
  icon?: FC<any>;
  id: string;
};

export function NavLink({
  id,
  link,
  label,
  subLabel,
  isRoot,
  icon: Icon,
}: NavLinkProps): JSX.Element {
  return (
    <Text<typeof Link>
      component={Link}
      className={["link", isRoot ? "rootLink" : "", Icon ? "hasIcon" : ""].join(
        " "
      )}
      to={link}
      key={id}
    >
      {Icon && (
        <ThemeIcon variant="light" size={30}>
          <Icon style={{ width: rem(18), height: rem(18) }} />
        </ThemeIcon>
      )}

      <Flex style={{ flexGrow: "1" }} className="label">
        <span>{label}</span>
        {subLabel && <span style={{ marginLeft: "auto" }}>{subLabel}</span>}
      </Flex>
    </Text>
  );
}
