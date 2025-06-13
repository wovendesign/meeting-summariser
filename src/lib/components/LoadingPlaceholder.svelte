<script lang="ts">
  interface Props {
    class?: string;
  }

  let { class: className = "" }: Props = $props();
</script>

<div class="flex-wrap gap-2 animate-pulse flex {className}">
  <div class="h-4 w-12 bg-foreground/10 rounded"></div>
  <div class="h-4 w-24 bg-foreground/10 rounded"></div>
  <div class="h-4 w-16 bg-foreground/10 rounded"></div>
  <div class="h-4 w-20 bg-foreground/10 rounded"></div>
  <div class="h-4 w-32 bg-foreground/10 rounded"></div>
  <div class="h-4 w-28 bg-foreground/10 rounded"></div>
  <div class="h-4 w-24 bg-foreground/10 rounded"></div>
  <div class="h-4 w-20 bg-foreground/10 rounded"></div>
  <div class="h-4 w-16 bg-foreground/10 rounded"></div>
  <div class="h-4 w-12 bg-foreground/10 rounded"></div>
  <div class="h-4 w-24 bg-foreground/10 rounded"></div>
  <div class="h-4 w-16 bg-foreground/10 rounded"></div>
  <div class="h-4 w-20 bg-foreground/10 rounded"></div>
  <div class="h-4 w-32 bg-foreground/10 rounded"></div>
  <div class="h-4 w-28 bg-foreground/10 rounded"></div>
  <div class="h-4 w-24 bg-foreground/10 rounded"></div>
</div>
