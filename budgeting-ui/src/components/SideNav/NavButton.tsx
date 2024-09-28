import { Box, rem, ThemeIcon } from "@mantine/core";
import { FC } from "react";
import "./LinksGroup/LinksGroup.css";

export type NavButtonProps = {
  onClick: () => void;
  label: string;
  subLabel?: string;
  isRoot?: boolean;
  icon?: FC<any>;
  id: string;
};

export function NavButton({
  id,
  onClick,
  label,
  subLabel,
  isRoot,
  icon: Icon,
}: NavButtonProps): JSX.Element {
  return (
    <Box<"button">
      className={[
        "button",
        isRoot ? "rootButton" : "",
        Icon ? "hasIcon" : "",
      ].join(" ")}
      key={id}
      onClick={onClick}
    >
      {Icon && (
        <ThemeIcon variant="light" size={30}>
          <Icon style={{ width: rem(18), height: rem(18) }} />
        </ThemeIcon>
      )}

      <Box display={"flex"} className="label">
        <span>{label}</span>
        {subLabel && <span style={{ marginLeft: "auto" }}>{subLabel}</span>}
      </Box>
    </Box>
  );
}
