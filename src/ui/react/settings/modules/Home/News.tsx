import { useInterval } from "libs/ui/react/utils/hooks";
import { cx } from "libs/ui/react/utils/styling";
import { Button, Skeleton } from "antd";
import { useAnimate } from "framer-motion";
import { useEffect, useState } from "react";
import { useTranslation } from "react-i18next";

import cs from "./News.module.css";

const PRODUCT_API_URL = "https://product.seelen.io";

interface BlogPromotion {
  type: "None" | "ShowEverywhere" | "ShowInApp";
  value?: string;
}

interface Blog {
  _id: string;
  slug: string;
  promotion: BlogPromotion;
  body: {
    title: string;
    description: string;
    portrait: string | null;
  };
}

interface New {
  title: string;
  message: string;
  url: string;
  image: string | null;
}

async function getPromotedNews(): Promise<New[]> {
  let response = await fetch(`${PRODUCT_API_URL}/blogs`);
  if (!response.ok) {
    return [];
  }
  let blogs: Blog[] = await response.json();

  return blogs
    .filter(
      (b) =>
        (b.promotion.type === "ShowInApp" && b.promotion.value === "seelen-ui") ||
        b.promotion.type === "ShowEverywhere",
    )
    .map((b) => ({
      title: b.body.title,
      message: b.body.description,
      url: `https://seelen.io/blog/${b.slug}`,
      image: b.body.portrait,
    }));
}

export function NoticeSlider() {
  const { t } = useTranslation();

  const discordBanner: New = {
    title: t("discord_banner.title"),
    message: t("discord_banner.message"),
    url: "https://discord.gg/ABfASx5ZAJ",
    image: "discord.webp",
  };

  const [news, setNews] = useState<New[]>([discordBanner]);
  const [currentIdx, setCurrentIdx] = useState<number>(0);

  const [scope, animate] = useAnimate<HTMLDivElement>();

  useEffect(() => {
    getPromotedNews().then((promoted) => setNews([discordBanner, ...promoted]));
  }, []);

  useInterval(
    () => {
      animate(scope.current, { opacity: 0 }).then(() => {
        setCurrentIdx((v) => v + 1);
        animate(scope.current, { opacity: 1 });
      });
    },
    10000,
    [currentIdx],
  );

  let current = news[currentIdx % news.length];

  return (
    <div className={cs.notices}>
      <div ref={scope} className={cs.notice}>
        {current
          ? (
            <>
              {current.image
                ? <img className={cs.image} src={current.image} alt={current.title} />
                : <div className={cs.image} />}
              <div className={cs.content}>
                <h3 className={cs.title}>{current.title}</h3>
                <p className={cs.message}>{current.message}</p>
                <div className={cs.linkButton}>
                  <Button href={current.url} target="_blank" type="primary">
                    {currentIdx === 0 ? t("join_us") : t("see_more")}
                  </Button>
                </div>
              </div>
            </>
          )
          : (
            <>
              <div className={cs.image} />
              <div className={cs.content}>
                <Skeleton active className={cs.title} paragraph={false} />
                <Skeleton active className={cs.message} title={false} />
                <div className={cs.linkButton}>
                  <Skeleton.Button active />
                </div>
              </div>
            </>
          )}
      </div>
      <div className={cs.pagination}>
        {news.map((_item, index) => (
          <div
            key={index}
            className={cx(cs.paginationDot, {
              [cs.active!]: index === currentIdx % news.length,
            })}
            onClick={() => {
              animate(scope.current, { opacity: 0 }).then(() => {
                setCurrentIdx(index);
                animate(scope.current, { opacity: 1 });
              });
            }}
          />
        ))}
      </div>
    </div>
  );
}
