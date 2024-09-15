import { Meta, StoryObj } from "@storybook/react";
import { SideNav } from "./SideNav";

type Story = StoryObj<typeof SideNav>;

export default {
  component: SideNav,
} satisfies Meta<typeof SideNav>;

export const Default: Story = {};
