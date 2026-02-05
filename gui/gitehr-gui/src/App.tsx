import {
  AppShell,
  Avatar,
  Badge,
  Box,
  Button,
  Card,
  Center,
  Divider,
  Group,
  Loader,
  NavLink,
  Stack,
  Text,
  Textarea,
  TextInput,
  ThemeIcon,
  Title,
  Alert,
} from "@mantine/core";
import {
  IconActivity,
  IconAlertCircle,
  IconCalendar,
  IconChartBar,
  IconFileText,
  IconFolderOpen,
  IconSearch,
  IconSettings,
  IconUser,
  IconPlus,
} from "@tabler/icons-react";
import { useEffect, useState } from "react";
import gitehrLogo from "./assets/gitehr-logo.svg";
import "./App.css";
import {
  addJournalEntry,
  getCurrentDir,
  getJournalEntries,
  getStateFiles,
  getStatus,
  isGitehrRepo,
  pickFolder,
  initRepo,
  type JournalEntryInfo,
  type RepoStatusInfo,
  type StateFileInfo,
} from "./api/gitehr";

function App() {
  const [repoPath, setRepoPath] = useState<string | null>(null);
  const [repoChecked, setRepoChecked] = useState(false);
  const [status, setStatus] = useState<RepoStatusInfo | null>(null);
  const [entries, setEntries] = useState<JournalEntryInfo[]>([]);
  const [stateFiles, setStateFiles] = useState<StateFileInfo[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [creating, setCreating] = useState(false);

  const [newEntryContent, setNewEntryContent] = useState("");
  const [submitting, setSubmitting] = useState(false);

  useEffect(() => {
    const checkInitialRepo = async () => {
      try {
        const cwd = await getCurrentDir();
        const isRepo = await isGitehrRepo(cwd);
        if (isRepo) {
          setRepoPath(cwd);
        }
      } catch (err) {
        console.error("Failed to check initial repo:", err);
      } finally {
        setRepoChecked(true);
      }
    };
    checkInitialRepo();
  }, []);

  const handlePickFolder = async () => {
    try {
      const folder = await pickFolder();
      if (folder) {
        const isRepo = await isGitehrRepo(folder);
        if (isRepo) {
          setRepoPath(folder);
          setError(null);
        } else {
          setError("Selected folder is not a GitEHR repository (no .gitehr directory found).");
        }
      }
    } catch (err) {
      console.error("Failed to pick folder:", err);
      setError("Failed to open folder picker.");
    }
  };

  const handleInitRepo = async () => {
    try {
      const folder = await pickFolder();
      if (folder) {
        setCreating(true);
        setError(null);
        try {
          await initRepo(folder);
          setRepoPath(folder);
        } catch (err) {
          console.error("Failed to init repo:", err);
          const message = typeof err === "string" ? err : String(err);
          if (message.includes("GitEHR CLI not found")) {
            setError(message);
          } else {
            setError("Failed to create repository: " + message);
          }
        } finally {
          setCreating(false);
        }
      }
    } catch (err) {
      console.error("Failed to pick folder:", err);
      setError("Failed to open folder picker.");
    }
  };

  const fetchData = async () => {
    if (!repoPath) return;
    setLoading(true);
    setError(null);
    try {
      const [statusData, entriesData, stateFilesData] = await Promise.all([
        getStatus(repoPath),
        getJournalEntries(repoPath, { limit: 10, reverse: true }),
        getStateFiles(repoPath),
      ]);
      setStatus(statusData);
      setEntries(entriesData);
      setStateFiles(stateFilesData);
    } catch (err) {
      console.error("Failed to fetch data:", err);
      setError(
        "Failed to load GitEHR data. Please ensure the backend is running and the repo path is correct."
      );
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    if (repoPath) {
      fetchData();
    }
  }, [repoPath]);

  const handleAddEntry = async () => {
    if (!repoPath || !newEntryContent.trim()) return;
    setSubmitting(true);
    try {
      await addJournalEntry(repoPath, newEntryContent);
      setNewEntryContent("");
      fetchData();
    } catch (err) {
      console.error("Failed to add entry:", err);
      alert("Failed to add entry: " + err);
    } finally {
      setSubmitting(false);
    }
  };

  const getStateContent = (filename: string) => {
    const file = stateFiles.find((f) => f.name === filename || f.name === filename + ".md");
    if (!file) return "Not documented";
    // Strip markdown headers if present (simple heuristic)
    return file.content.replace(/^#+\s+/gm, "").trim();
  };

  // Loading state while checking initial repo
  if (!repoChecked) {
    return (
      <Center h="100vh">
        <Stack align="center" gap="md">
          <Loader size="lg" color="teal" />
          <Text c="dimmed">Checking for GitEHR repository...</Text>
        </Stack>
      </Center>
    );
  }

  // No repo detected - show folder picker
  if (!repoPath) {
    return (
      <Center h="100vh" bg="gray.0">
        <Card radius="lg" shadow="md" p="xl" w={400}>
          <Stack align="center" gap="lg">
            <Box className="brand-mark" style={{ width: 80, height: 80 }}>
              <img src={gitehrLogo} alt="GitEHR logo" style={{ width: "100%", height: "100%" }} />
            </Box>
            <Stack align="center" gap="xs">
              <Title order={3}>No GitEHR Repository Detected</Title>
              <Text size="sm" c="dimmed" ta="center">
                Select a folder containing a GitEHR repository to get started.
              </Text>
            </Stack>
            {error && (
              <Alert
                icon={<IconAlertCircle size={16} />}
                color={error.includes("GitEHR CLI not found") ? "yellow" : "red"}
                w="100%"
                onClose={() => setError(null)}
                withCloseButton
              >
                {error}
              </Alert>
            )}
            <Button
              size="md"
              leftSection={<IconFolderOpen size={18} />}
              onClick={handlePickFolder}
              fullWidth
            >
              Open Repository
            </Button>
            <Button
              size="md"
              variant="light"
              leftSection={<IconPlus size={18} />}
              onClick={handleInitRepo}
              loading={creating}
              fullWidth
            >
              Create New Repository
            </Button>
          </Stack>
        </Card>
      </Center>
    );
  }

  // Normal app view when repo is loaded
  return (
    <AppShell
      className="app-shell"
      header={{ height: 64, offset: true }}
      navbar={{ width: 260, breakpoint: "sm" }}
      aside={{ width: 320, breakpoint: "md" }}
      padding="md"
    >
      <AppShell.Header className="app-header">
        <Group h="100%" px="md" justify="space-between">
          <Group gap="sm">
            <Box className="brand-mark">
              <img src={gitehrLogo} alt="GitEHR logo" />
            </Box>
            <Box>
              <Text size="xs" c="dimmed" fw={600} tt="uppercase">
                GitEHR Reference GUI
              </Text>
              <Title order={4} className="brand-title">
                St. Aria Health
              </Title>
            </Box>
          </Group>
          <Group gap="sm">
            <TextInput
              placeholder="Search patients, visits, or tags"
              leftSection={<IconSearch size={16} />}
              size="sm"
              className="search-input"
            />
          </Group>
        </Group>
      </AppShell.Header>

      <AppShell.Navbar className="app-sidebar">
        <Stack gap="md" p="md">
          <Text size="xs" tt="uppercase" fw={600} c="dimmed">
            Navigation
          </Text>
          <NavLink
            label="Patients"
            leftSection={<IconUser size={18} />}
            active
          />
          <NavLink label="Appointments" leftSection={<IconCalendar size={18} />} />
          <NavLink label="Reports" leftSection={<IconChartBar size={18} />} />
          <NavLink label="Journal" leftSection={<IconFileText size={18} />} />
          <NavLink label="Vitals" leftSection={<IconActivity size={18} />} />
          <Divider />
          <NavLink label="Settings" leftSection={<IconSettings size={18} />} />
          <Card className="sidebar-card" radius="lg" mt="md">
            <Stack gap={6}>
              <Text size="xs" tt="uppercase" fw={600} c="dimmed">
                Repo Status
              </Text>
              {status && (
                <>
                  <Group justify="space-between">
                    <Text size="sm">Entries</Text>
                    <Badge variant="light" color="teal">
                      {status.journal_entry_count}
                    </Badge>
                  </Group>
                  <Group justify="space-between">
                    <Text size="sm">Encrypted</Text>
                    <Badge
                      variant="light"
                      color={status.is_encrypted ? "green" : "gray"}
                    >
                      {status.is_encrypted ? "Yes" : "No"}
                    </Badge>
                  </Group>
                  <Group justify="space-between">
                    <Text size="sm">Version</Text>
                    <Text size="sm" c="dimmed">
                      {status.gitehr_version || "N/A"}
                    </Text>
                  </Group>
                </>
              )}
            </Stack>
          </Card>
        </Stack>
      </AppShell.Navbar>

      <AppShell.Main className="app-main">
        <Box className="main-surface">
          <Stack gap="md">
            {error && (
              <Alert
                icon={<IconAlertCircle size={16} />}
                title="Error"
                color="red"
                withCloseButton
                onClose={() => setError(null)}
              >
                {error}
              </Alert>
            )}
            <Group justify="space-between" align="flex-end">
            <Box>
              <Title order={2}>Patient Overview</Title>
              <Text size="sm" c="dimmed">
                Timeline and summaries from the linked GitEHR repository.
              </Text>
            </Box>
            <Group gap="xs">
              <Badge size="lg" variant="light" color="teal">
                In clinic
              </Badge>
              <Badge size="lg" variant="light" color="gray">
                MRN 231-889
              </Badge>
            </Group>
          </Group>

          <Card radius="lg" className="panel-card">
              <Group justify="space-between" mb="md">
                <Group gap="xs">
                  <ThemeIcon variant="light" color="teal">
                    <IconFileText size={18} />
                  </ThemeIcon>
                  <Text fw={600}>Journal</Text>
                </Group>
                <Badge variant="outline">{entries.length} entries</Badge>
              </Group>

              <Group align="flex-start" gap="sm" mb="md">
                <Textarea
                  placeholder="Write a new journal entry..."
                  style={{ flex: 1 }}
                  minRows={2}
                  maxRows={4}
                  autosize
                  value={newEntryContent}
                  onChange={(e) => setNewEntryContent(e.currentTarget.value)}
                  disabled={submitting}
                />
                <Button
                  onClick={handleAddEntry}
                  loading={submitting}
                  disabled={!newEntryContent.trim()}
                >
                  Add
                </Button>
              </Group>

              <Divider mb="md" />

              {loading ? (
                <Center py="md">
                  <Loader size="sm" />
                </Center>
              ) : entries.length === 0 ? (
                <Text size="sm" c="dimmed" ta="center" py="md">
                  No journal entries yet. Add your first entry above.
                </Text>
              ) : (
                <Stack gap="sm">
                  {entries.map((entry, i) => (
                    <Card key={i} withBorder padding="sm" radius="md">
                      <Text size="sm">{entry.content_preview}</Text>
                      <Text size="xs" c="dimmed" mt="xs">
                        {new Date(entry.timestamp).toLocaleString()} · {entry.author || "Unknown"}
                      </Text>
                    </Card>
                  ))}
                </Stack>
              )}
            </Card>
          </Stack>
        </Box>
      </AppShell.Main>

      <AppShell.Aside className="app-aside">
        <Stack gap="md" p="md">
          <Card radius="lg" className="panel-card">
            <Group>
              <Avatar radius="xl" size="lg" color="teal">
                JP
              </Avatar>
              <Box>
                <Text fw={600}>Jamie Porter</Text>
                <Text size="sm" c="dimmed">
                  DOB 1986-05-14
                </Text>
              </Box>
            </Group>
            <Group gap="xs" mt="md">
              <Badge variant="light">Cardiology</Badge>
              <Badge variant="light">Post-op</Badge>
              <Badge variant="light">Remote care</Badge>
            </Group>
          </Card>

          <Card radius="lg" className="panel-card">
            <Text size="xs" tt="uppercase" fw={600} c="dimmed" mb="xs">
              Stateful summary
            </Text>
            <Stack gap="sm">
              {loading ? (
                <Loader size="sm" />
              ) : (
                <>
                  <Box>
                    <Text size="sm" fw={600}>
                      Allergies
                    </Text>
                    <Text size="sm" c="dimmed">
                      {getStateContent("allergies")}
                    </Text>
                  </Box>
                  <Box>
                    <Text size="sm" fw={600}>
                      Current medications
                    </Text>
                    <Text size="sm" c="dimmed">
                      {getStateContent("medications")}
                    </Text>
                  </Box>
                  <Box>
                    <Text size="sm" fw={600}>
                      Demographics
                    </Text>
                    <Text size="sm" c="dimmed">
                      {getStateContent("demographics")}
                    </Text>
                  </Box>
                </>
              )}
            </Stack>
          </Card>

            <Card radius="lg" className="panel-card">
              <Text size="xs" tt="uppercase" fw={600} c="dimmed" mb="xs">
                Activity feed
              </Text>
              <Stack gap="sm">
                {loading ? (
                  <Loader size="sm" />
                ) : (
                  entries.slice(0, 3).map((entry, i) => (
                    <Group align="flex-start" key={i}>
                      <ThemeIcon variant="light" color="teal">
                        <IconFileText size={16} />
                      </ThemeIcon>
                      <Box>
                        <Text size="sm">
                          Journal entry by {entry.author || "Unknown"}
                        </Text>
                        <Text size="xs" c="dimmed">
                          {new Date(entry.timestamp).toLocaleTimeString([], {
                            hour: "2-digit",
                            minute: "2-digit",
                          })}
                        </Text>
                      </Box>
                    </Group>
                  ))
                )}
              </Stack>
            </Card>
        </Stack>
      </AppShell.Aside>
    </AppShell>
  );
}

export default App;
