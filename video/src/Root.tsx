import { Composition, Series, Audio, staticFile } from "remotion";
import { useEffect, useState } from "react";
import { Title } from "./scenes/Title";
import { Problem } from "./scenes/Problem";
import { Capture } from "./scenes/Capture";
import { FeedAgent } from "./scenes/FeedAgent";
import { MCP } from "./scenes/MCP";
import { Close } from "./scenes/Close";

const FPS = 30;
const WIDTH = 1920;
const HEIGHT = 1080;

const SCENES: ReadonlyArray<{
  name: string;
  component: React.ComponentType;
  durationInFrames: number;
}> = [
  { name: "Title", component: Title, durationInFrames: 5 * FPS },
  { name: "Problem", component: Problem, durationInFrames: 15 * FPS },
  { name: "Capture", component: Capture, durationInFrames: 20 * FPS },
  { name: "FeedAgent", component: FeedAgent, durationInFrames: 20 * FPS },
  { name: "MCP", component: MCP, durationInFrames: 20 * FPS },
  { name: "Close", component: Close, durationInFrames: 10 * FPS },
];

const totalDuration = SCENES.reduce(
  (sum, scene) => sum + scene.durationInFrames,
  0,
);

// Audio bed is optional. Drop a royalty-free track at
// `video/public/audio/bed.mp3` and it will auto-wire. See README.
const AudioBed: React.FC = () => {
  const [hasBed, setHasBed] = useState(false);
  useEffect(() => {
    fetch(staticFile("audio/bed.mp3"), { method: "HEAD" })
      .then((r) => setHasBed(r.ok))
      .catch(() => setHasBed(false));
  }, []);
  if (!hasBed) return null;
  return <Audio src={staticFile("audio/bed.mp3")} volume={0.35} />;
};

const SpikesDemo: React.FC = () => (
  <>
    <AudioBed />
    <Series>
      {SCENES.map((scene) => (
        <Series.Sequence
          key={scene.name}
          durationInFrames={scene.durationInFrames}
        >
          <scene.component />
        </Series.Sequence>
      ))}
    </Series>
  </>
);

export const Root: React.FC = () => (
  <>
    <Composition
      id="SpikesDemo"
      component={SpikesDemo}
      durationInFrames={totalDuration}
      fps={FPS}
      width={WIDTH}
      height={HEIGHT}
    />
    <Composition
      id="SpikesDemoLoop"
      component={Close}
      durationInFrames={10 * FPS}
      fps={FPS}
      width={WIDTH}
      height={HEIGHT}
    />
  </>
);
