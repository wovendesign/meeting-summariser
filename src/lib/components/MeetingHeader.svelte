<script lang="ts">
  import { buttonVariants } from "$lib/components/ui/button/button.svelte";
  import * as DropdownMenu from "$lib/components/ui/dropdown-menu";
  import * as Dialog from "$lib/components/ui/dialog/index.js";
  import Button from "$lib/components/ui/button/button.svelte";
  import { Input } from "$lib/components/ui/input";
  import { Label } from "$lib/components/ui/label";
  import Ellipsis from "@lucide/svelte/icons/ellipsis";
  import Pen from "@lucide/svelte/icons/pen";
  import FolderClosed from "@lucide/svelte/icons/folder-closed";
  import clsx from "clsx";

  interface Props {
    name: string;
    generatingName: boolean;
    onRevealInFinder?: () => void;
  }

  let { name, generatingName, onRevealInFinder }: Props = $props();
</script>

<div class="flex items-center justify-between">
  <h2 class={clsx("text-2xl font-bold", generatingName && "animate-pulse")}>
    {name}
  </h2>
  <DropdownMenu.Root>
    <DropdownMenu.Trigger
      class={buttonVariants({ variant: "outline", size: "icon" })}
    >
      <Ellipsis />
    </DropdownMenu.Trigger>
    <DropdownMenu.Content class="w-56 mr-4">
      <DropdownMenu.Item>
        <Pen />
        <span>Rename Meeting</span>
        <Dialog.Root>
          <Dialog.Trigger class={buttonVariants({ variant: "outline" })}
            >Edit Profile</Dialog.Trigger
          >
          <Dialog.Content class="sm:max-w-[425px]">
            <Dialog.Header>
              <Dialog.Title>Edit profile</Dialog.Title>
              <Dialog.Description>
                Make changes to your profile here. Click save when you're done.
              </Dialog.Description>
            </Dialog.Header>
            <div class="grid gap-4 py-4">
              <div class="grid grid-cols-4 items-center gap-4">
                <Label for="name" class="text-right">Name</Label>
                <Input id="name" value="Pedro Duarte" class="col-span-3" />
              </div>
              <div class="grid grid-cols-4 items-center gap-4">
                <Label for="username" class="text-right">Username</Label>
                <Input id="username" value="@peduarte" class="col-span-3" />
              </div>
            </div>
            <Dialog.Footer>
              <Button type="submit">Save changes</Button>
            </Dialog.Footer>
          </Dialog.Content>
        </Dialog.Root>
      </DropdownMenu.Item>
      <DropdownMenu.Item onclick={onRevealInFinder}>
        <FolderClosed />
        <span>Reveal in Finder</span>
      </DropdownMenu.Item>
    </DropdownMenu.Content>
  </DropdownMenu.Root>
</div>
