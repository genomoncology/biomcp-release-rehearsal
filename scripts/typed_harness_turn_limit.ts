// Bound the existing default pi client without changing provider or model.
export default function (pi: { on: (event: string, handler: () => void) => void }) {
  let requests = 0;
  pi.on("before_provider_request", () => {
    if (++requests > 3) {
      process.stderr.write("typed harness: three model requests exhausted\n");
      process.exit(2);
    }
  });
}
