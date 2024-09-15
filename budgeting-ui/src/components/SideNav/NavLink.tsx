import { rem, Text, ThemeIcon } from "@mantine/core";
import { FC } from "react";
import { Link } from "react-router-dom";

export type NavLinkProps = {
  link: string;
  label: string;
  endLabel?: string;
  isRoot?: boolean;
  icon?: FC<any>;
};

export function NavLink({
  link,
  label,
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
      key={label}
    >
      {Icon && (
        <ThemeIcon variant="light" size={30}>
          <Icon style={{ width: rem(18), height: rem(18) }} />
        </ThemeIcon>
      )}

      <span className="label">{label}</span>
    </Text>
  );
}
