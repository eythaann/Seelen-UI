import { Icon } from "libs/ui/react/components/Icon/index.tsx";
import { ResourceText } from "libs/ui/react/components/ResourceText/index.tsx";
import { Button, Flex, Modal } from "antd";
import { useState } from "preact/hooks";
import { useTranslation } from "react-i18next";

import { compressUuid, ResourceIcon } from "./ResourceCard.tsx";
import { resourcesWithUpdate } from "../../state/resources.ts";
import cs from "./updates.module.css";

export function ResourceUpdatesModal() {
  const [dismissed, setDismissed] = useState(false);
  const { t } = useTranslation();

  const updates = resourcesWithUpdate.value;
  const open = !dismissed && updates.length > 0;

  return (
    <Modal
      open={open}
      onCancel={() => setDismissed(true)}
      title={
        <Flex align="center" gap={8}>
          <Icon iconName="MdUpdate" />
          {t("resources.updates_available")}
        </Flex>
      }
      footer={null}
      centered
    >
      <p>{t("resources.updates_available_description")}</p>
      <Flex vertical gap={8} className={cs.list}>
        {updates.map((resource) => (
          <Flex key={resource.id} align="center" justify="space-between" gap={12} className={cs.item}>
            <Flex align="center" gap={8}>
              <ResourceIcon kind={resource.kind} />
              <ResourceText text={resource.metadata.displayName} />
            </Flex>
            <Button
              type="primary"
              href={`https://seelen.io/resources/${compressUuid(resource.id)}?update`}
              target="_blank"
              onClick={() =>
                setDismissed(true)}
            >
              <Icon iconName="MdUpdate" />
              {t("resources.update_now")}
            </Button>
          </Flex>
        ))}
      </Flex>
    </Modal>
  );
}
