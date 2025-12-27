<template>
  <Card class="relative w-[600px] text-white bg-[#171717] border-none">
    <CardContent class="p-4">
      <pre class="overflow-x-auto text-sm"><code>{{ text }}</code></pre>

      <Button
        variant="outline"
        size="icon"
        class="absolute top-2 right-2 border-none bg-red hover:bg-[#222222] hover:text-white hover:cursor-pointer"
        @click="copy"
      >
        <Copy class="h-4 w-4" v-if="!copied" />
        <Check class="h-4 w-4 text-green-600" v-else />
      </Button>
    </CardContent>
  </Card>
</template>

<script setup lang="ts">
import { ref } from "vue";
import { Card, CardContent } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Copy, Check } from "lucide-vue-next";

const props = defineProps({
  text: {
    type: String,
    required: true,
  },
});

const copied = ref(false);

const copy = async () => {
  await navigator.clipboard.writeText(props.text);
  copied.value = true;
  setTimeout(() => (copied.value = false), 1200);
};
</script>
