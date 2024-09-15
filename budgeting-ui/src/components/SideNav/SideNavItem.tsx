import { Box, Group, rem, ThemeIcon, UnstyledButton } from "@mantine/core";
import { IconChevronRight } from "@tabler/icons-react";
import { FC, Fragment, ReactNode } from "react";

export type SideNavItemProps = {
  label: string;
  icon?: FC<any>;
  openable?: boolean;
  open?: boolean;
  onOpenToggled?: () => void;
  className?: string
};
export function SideNavItem({
  icon: Icon,
  label,
  openable,
  open,
  onOpenToggled,
  className
}: SideNavItemProps): JSX.Element {
  function Button({ children }: { children: ReactNode }) {
    return (
      <UnstyledButton className={className} onClick={() => onOpenToggled?.()}>
        {children}
      </UnstyledButton>
    );
  }

  const Wrapper = openable ? Button : Fragment;

  return (
    <Wrapper>
      <Group justify="space-between" gap={0}>
        <Box style={{ display: "flex", alignItems: "center" }}>
          {Icon && (
            <ThemeIcon variant="light" size={30}>
              <Icon style={{ width: rem(18), height: rem(18) }} />
            </ThemeIcon>
          )}
          <Box ml="md">{label}</Box>
        </Box>
        {openable && (
          <IconChevronRight
            className={"chevron"}
            stroke={1.5}
            style={{
              width: rem(16),
              height: rem(16),
              transform: open ? "rotate(-90deg)" : "none",
            }}
          />
        )}
      </Group>
    </Wrapper>
  );
}
