import { Resource } from "@seelen-ui/lib/types";
import { ResourceText } from "@shared/components/ResourceText";
import { Skeleton } from "antd";
import { useEffect, useState } from "react";
import { useTranslation } from "react-i18next";

import cs from "./MiniStore.module.css";

export function ResourceSkeleton() {
  return (
    <div className={cs.resourceSkeleton}>
      <Skeleton.Image active style={{ width: "100%", aspectRatio: "1 / 1" }} />
      <Skeleton active paragraph={false} />
    </div>
  );
}

interface Featured {
  newArrivals: Resource[];
  top: Resource[];
  popular: Resource[];
  staffLiked: Resource[];
}

export function RemoteResources() {
  const [resources, setResources] = useState<Resource[]>([]);

  const { t } = useTranslation();

  useEffect(() => {
    fetch("https://product.seelen.io/resources/featured")
      .then((res) => res.json())
      .then((data: Featured) =>
        setResources(
          data.newArrivals.filter((r) => {
            return !r.metadata.portrait?.includes("cloudinary");
          }),
        )
      )
      .catch(() => {});
  }, []);

  return (
    <>
      <h1 className={cs.title}>{t("home.new_resources")}</h1>
      <div className={cs.resources}>
        {resources.length === 0 &&
          Array.from({ length: 10 }).map((_, i) => (
            <ResourceSkeleton
              key={i}
            />
          ))}

        {resources.map((resource) => {
          if (!resource.metadata.portrait) return null;
          return (
            <a
              key={resource.id}
              href={`https://seelen.io/resources/${resource.friendlyId.replace("@", "")}`}
              target="_blank"
              className={cs.resource}
            >
              <img src={resource.metadata.portrait} />
              <div className={cs.text}>
                <ResourceText text={resource.metadata.displayName} />
              </div>
            </a>
          );
        })}
      </div>
    </>
  );
}
