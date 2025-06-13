<script lang="ts">
  interface Props {
    currentChunk: number;
    totalChunks: number;
    label: string;
    color?: string;
  }

  let {
    currentChunk,
    totalChunks,
    label,
    color = "bg-primary",
  }: Props = $props();

  const percentage = $derived(() => {
    if (totalChunks === 0) return 0;
    return Math.round((currentChunk / totalChunks) * 100);
  });
</script>

<div class="space-y-2">
  <div class="flex justify-between text-sm">
    <span>{label} {currentChunk} of {totalChunks}</span>
    <span>{percentage()}%</span>
  </div>
  <div class="w-full bg-muted rounded-full h-2">
    <div
      class="{color} h-2 rounded-full transition-all duration-300 ease-in-out"
      style="width: {percentage()}%"
    ></div>
  </div>
</div>
