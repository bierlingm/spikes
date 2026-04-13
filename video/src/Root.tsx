import { Composition, Series } from "remotion";
import { Title } from "./scenes/Title";
import { Problem } from "./scenes/Problem";

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
  // Scenes 2-5 are storyboarded in STORYBOARD.md. Stubs land in src/scenes/
  // as they're built out, then get appended here.
];

const totalDuration = SCENES.reduce(
  (sum, scene) => sum + scene.durationInFrames,
  0,
);

const SpikesDemo: React.FC = () => {
  return (
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
  );
};

export const Root: React.FC = () => {
  return (
    <Composition
      id="SpikesDemo"
      component={SpikesDemo}
      durationInFrames={totalDuration}
      fps={FPS}
      width={WIDTH}
      height={HEIGHT}
    />
  );
};
