import { Button, Flex, Modal } from "antd";
import { invoke, SeelenCommand } from "@seelen-ui/lib";
import { z } from "zod";

import cs from "./index.module.css";
import { useState } from "preact/hooks";
import { useTranslation } from "react-i18next";
import { Icon } from "libs/ui/react/components/Icon";
import { cx } from "libs/ui/react/utils/styling";
import { persistentSignal } from "libs/ui/react/utils/PersistentSignal";

const REVIEW_URL = "ms-windows-store://review/?ProductId=9P67C2D4T9FB&mode=mini";
const ONE_WEEK_MS = 7 * 24 * 60 * 60 * 1000;

const WelcomeStateSchema = z.object({
  welcomeShown: z.boolean().default(false),
  alreadyReviewed: z.boolean().default(false),
  lastReviewPrompt: z.number().nullish(),
});

type WelcomeState = z.infer<typeof WelcomeStateSchema>;
const DEFAULT_STATE = WelcomeStateSchema.parse({});

const ReviewState = await persistentSignal("welcomeModal", DEFAULT_STATE, WelcomeStateSchema);

function shouldShowReviewModal(state: WelcomeState): boolean {
  if (state.alreadyReviewed) {
    return false;
  }

  if (!state.lastReviewPrompt) {
    ReviewState.value = { ...ReviewState.value, lastReviewPrompt: Date.now() };
    return false;
  }

  return Date.now() - state.lastReviewPrompt >= ONE_WEEK_MS;
}

function StarRating({ onRate }: { onRate: () => void }) {
  const [hovered, setHovered] = useState(-1);

  return (
    <div className={cs.starRating}>
      {[0, 1, 2, 3, 4].map((index) => (
        <Icon
          key={index}
          className={cx(cs.star, { [cs.filled!]: index <= hovered })}
          iconName="FaStar"
          onMouseEnter={() => setHovered(index)}
          onMouseLeave={() => setHovered(-1)}
          onClick={onRate}
          size={32}
        />
      ))}
    </div>
  );
}

export const WelcomeModal = () => {
  const state = ReviewState.value;
  const isNewUser = !state.welcomeShown;
  const showReviewModal = !isNewUser && shouldShowReviewModal(state);

  const { t } = useTranslation();

  const openReviewAndMark = () => {
    invoke(SeelenCommand.OpenFile, { path: REVIEW_URL });
    ReviewState.value = { ...ReviewState.value, alreadyReviewed: true, welcomeShown: true };
  };

  return (
    <>
      <Modal
        className={cs.welcome}
        open={isNewUser}
        centered
        closable={false}
        title={`🎉 ${t("welcome.title")} 🥳`}
        footer={
          <Flex justify="flex-end" gap="1rem">
            <Button type="link" onClick={openReviewAndMark}>
              {t("welcome.give_a_review")} 😉
            </Button>
            <Button
              type="primary"
              onClick={() => {
                ReviewState.value = { ...ReviewState.value, welcomeShown: true };
              }}
            >
              {t("welcome.ok")}
            </Button>
          </Flex>
        }
      >
        <p>{t("welcome.message")}</p>
        <br />
        <p>{t("welcome.review")}</p>
        <br />
      </Modal>

      <Modal
        className={cs.welcome}
        open={showReviewModal}
        centered
        closable={false}
        title={`😄 ${t("review_request.title")}`}
        footer={
          <Flex justify="flex-end" gap="1rem">
            <Button
              onClick={() => {
                ReviewState.value = { ...ReviewState.value, lastReviewPrompt: Date.now() };
              }}
            >
              {t("review_request.not_now")}
            </Button>
            <Button type="primary" onClick={openReviewAndMark} style={{ minWidth: "70px" }}>
              {t("review_request.sure")}
            </Button>
          </Flex>
        }
      >
        <p>{t("welcome.review")}</p>
        <StarRating onRate={openReviewAndMark} />
      </Modal>
    </>
  );
};
