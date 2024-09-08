import { Button, Modal } from "@mantine/core";
import { useState } from "react";
import { ConfigurePayPeriod } from "../ConfigurePayPeriod/ConfigurePayPeriod";

export function AppBar(): JSX.Element {
  const [showConfigurePayPeriod, setShowConfigurePayPeriod] = useState(false);

  return (
    <div>
      <Button onClick={() => setShowConfigurePayPeriod(true)}>
        Configure Pay Period
      </Button>
      <Modal
        opened={showConfigurePayPeriod}
        onClose={() => setShowConfigurePayPeriod(false)}
      >
        <ConfigurePayPeriod />
      </Modal>
    </div>
  );
}
