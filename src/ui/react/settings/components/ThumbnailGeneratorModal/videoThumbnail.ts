/**
 * Extracts the first frame of a video as a thumbnail image
 * @param videoElement - The video element to extract the frame from
 * @param quality - JPEG quality (0-1), default 0.9
 * @returns Promise with the thumbnail as Uint8Array bytes
 */
export async function extractVideoThumbnail(
  videoElement: HTMLVideoElement,
  quality: number = 0.9,
): Promise<Uint8Array | null> {
  try {
    // Wait for video to load metadata if not already loaded
    if (videoElement.readyState < HTMLMediaElement.HAVE_METADATA) {
      await new Promise<void>((resolve, reject) => {
        const handleLoaded = () => {
          videoElement.removeEventListener("loadedmetadata", handleLoaded);
          videoElement.removeEventListener("error", handleError);
          resolve();
        };
        const handleError = () => {
          videoElement.removeEventListener("loadedmetadata", handleLoaded);
          videoElement.removeEventListener("error", handleError);
          reject(new Error("Failed to load video metadata"));
        };
        videoElement.addEventListener("loadedmetadata", handleLoaded);
        videoElement.addEventListener("error", handleError);
      });
    }

    // Seek to first frame (1 second to ensure we get a good frame)
    videoElement.currentTime = 1;

    // Wait for seek to complete
    await new Promise<void>((resolve, reject) => {
      const handleSeeked = () => {
        videoElement.removeEventListener("seeked", handleSeeked);
        videoElement.removeEventListener("error", handleError);
        resolve();
      };
      const handleError = () => {
        videoElement.removeEventListener("seeked", handleSeeked);
        videoElement.removeEventListener("error", handleError);
        reject(new Error("Failed to seek video"));
      };
      videoElement.addEventListener("seeked", handleSeeked);
      videoElement.addEventListener("error", handleError);
    });

    // Create canvas with video dimensions
    const canvas = document.createElement("canvas");
    canvas.width = videoElement.videoWidth;
    canvas.height = videoElement.videoHeight;

    // Draw video frame to canvas
    const ctx = canvas.getContext("2d");
    if (!ctx) {
      throw new Error("Failed to get canvas 2D context");
    }

    ctx.drawImage(videoElement, 0, 0, canvas.width, canvas.height);

    // Convert canvas to blob
    const blob = await new Promise<Blob | null>((resolve) => {
      canvas.toBlob((blob) => resolve(blob), "image/jpeg", quality);
    });

    if (!blob) {
      throw new Error("Failed to create thumbnail blob");
    }

    // Convert blob to Uint8Array for efficient transfer to Rust
    const arrayBuffer = await blob.arrayBuffer();
    return new Uint8Array(arrayBuffer);
  } catch (error) {
    console.error("Failed to extract video thumbnail:", error);
    return null;
  }
}

/**
 * Creates a hidden video element to extract thumbnail from a video source
 * @param videoSrc - The video source URL
 * @param quality - JPEG quality (0-1), default 0.9
 * @returns Promise with the thumbnail as Uint8Array bytes
 */
export async function extractThumbnailFromSource(
  videoSrc: string,
  quality: number = 0.9,
): Promise<Uint8Array | null> {
  const video = document.createElement("video");
  video.style.display = "none";
  video.crossOrigin = "anonymous";
  video.preload = "metadata";
  video.src = videoSrc;

  document.body.appendChild(video);

  try {
    const thumbnail = await extractVideoThumbnail(video, quality);
    return thumbnail;
  } finally {
    // Cleanup
    video.pause();
    video.removeAttribute("src");
    video.load();
    document.body.removeChild(video);
  }
}
