<script lang="ts">
  import { page } from '$app/state';
  import ViewSecret from './ViewSecret.svelte';
</script>

<ViewSecret id={page.params.id} />
