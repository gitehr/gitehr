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
import gitehrLogo from "../../assets/images/gitehr-logo-1000px-black-trans.png";
import "./App.css";
import {
  addJournalEntry,
  getCurrentDir,
  getJournalEntries,
  getStateFiles,
  getStatus,
  isGitehrRepo,
  hasMpi,
  getMpi,
  pickFolder,
  initStoreRoot,
  addStoreSubject,
  type JournalEntryInfo,
  type RepoStatusInfo,
  type StateFileInfo,
  type MpiInfo,
  type MpiPatientInfo,
} from "./api/gitehr";

function App() {
  const [repoPath, setRepoPath] = useState<string | null>(null);
  const [storeRoot, setStoreRoot] = useState<string | null>(null);
  const [repoChecked, setRepoChecked] = useState(false);
  const [status, setStatus] = useState<RepoStatusInfo | null>(null);
  const [entries, setEntries] = useState<JournalEntryInfo[]>([]);
  const [stateFiles, setStateFiles] = useState<StateFileInfo[]>([]);
  const [loading, setLoading] = useState(true);
  const [mpiLoading, setMpiLoading] = useState(false);
  const [mpi, setMpi] = useState<MpiInfo | null>(null);
  const [patientSearch, setPatientSearch] = useState("");
  const [error, setError] = useState<string | null>(null);
  const [creating, setCreating] = useState(false);
  const [firstSubjectName, setFirstSubjectName] = useState("");
  const [newSubjectName, setNewSubjectName] = useState("");

  const [newEntryContent, setNewEntryContent] = useState("");
  const [submitting, setSubmitting] = useState(false);

  const loadMpi = async (path: string) => {
    setMpiLoading(true);
    setError(null);
    try {
      const mpiData = await getMpi(path);
      setMpi(mpiData);
      setStoreRoot(mpiData.store_root);
      setRepoPath(null);
    } catch (err) {
      console.error("Failed to load MPI:", err);
      setError("Failed to load MPI data. Please ensure gitehr-mpi.json is valid.");
    } finally {
      setMpiLoading(false);
    }
  };

  const selectLatestPatientRepo = (mpiData: MpiInfo) => {
    if (mpiData.patients.length === 0) {
      setError("Store contains no subjects.");
      return;
    }
    const latest = mpiData.patients[mpiData.patients.length - 1];
    setRepoPath(latest.repo_path);
  };

  useEffect(() => {
    const checkInitialRepo = async () => {
      try {
        const cwd = await getCurrentDir();
        const isRepo = await isGitehrRepo(cwd);
        if (isRepo) {
          setRepoPath(cwd);
          return;
        }
        const hasIndex = await hasMpi(cwd);
        if (hasIndex) {
          await loadMpi(cwd);
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
          return;
        }
        const hasIndex = await hasMpi(folder);
        if (hasIndex) {
          await loadMpi(folder);
          setError(null);
        } else {
          setError(
            "Selected folder is not a GitEHR repository or store root (no .gitehr or gitehr-mpi.json found)."
          );
        }
      }
    } catch (err) {
      console.error("Failed to pick folder:", err);
      setError("Failed to open folder picker.");
    }
  };

  const handleCreateStore = async () => {
    try {
      const folder = await pickFolder();
      if (folder) {
        setCreating(true);
        setError(null);
        try {
          await initStoreRoot(folder, firstSubjectName);
          const mpiData = await getMpi(folder);
          setMpi(mpiData);
          setStoreRoot(mpiData.store_root);
          setFirstSubjectName("");
          selectLatestPatientRepo(mpiData);
        } catch (err) {
          console.error("Failed to init store root:", err);
          const message = typeof err === "string" ? err : String(err);
          if (message.includes("GitEHR CLI not found")) {
            setError(message);
          } else {
            setError("Failed to create store root: " + message);
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

  const handleAddSubject = async () => {
    if (!storeRoot) return;
    setCreating(true);
    setError(null);
    try {
      await addStoreSubject(storeRoot, newSubjectName);
      const mpiData = await getMpi(storeRoot);
      setMpi(mpiData);
      setStoreRoot(mpiData.store_root);
      setNewSubjectName("");
      selectLatestPatientRepo(mpiData);
    } catch (err) {
      console.error("Failed to add store subject:", err);
      const message = typeof err === "string" ? err : String(err);
      setError("Failed to add subject: " + message);
    } finally {
      setCreating(false);
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
      setError("Failed to add journal entry: " + err);
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

  const filteredPatients: MpiPatientInfo[] = mpi
    ? mpi.patients.filter((patient) => {
        if (!patientSearch.trim()) return true;
        const needle = patientSearch.toLowerCase();
        if (patient.patient_id.toLowerCase().includes(needle)) return true;
        if (patient.repo_path.toLowerCase().includes(needle)) return true;
        return patient.identifiers.some(
          (id) =>
            id.type.toLowerCase().includes(needle) ||
            id.value.toLowerCase().includes(needle)
        );
      })
    : [];

  const selectedPatient = mpi?.patients.find((patient) => patient.repo_path === repoPath);
  const selectedSubjectName =
    selectedPatient?.repo_path.split(/[\\/]/).filter(Boolean).pop() ||
    repoPath?.split(/[\\/]/).filter(Boolean).pop() ||
    "Open record";

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

  // No repo detected and no MPI detected - show folder picker
  if (!repoPath && !storeRoot) {
    return (
      <Center h="100vh" bg="gray.0">
        <Card radius="md" shadow="sm" p="xl" w={400}>
          <Stack align="center" gap="lg">
            <Box className="brand-mark" style={{ width: 80, height: 80 }}>
              <img src={gitehrLogo} alt="GitEHR logo" style={{ width: "100%", height: "100%" }} />
            </Box>
            <Stack align="center" gap="xs">
              <Title order={3}>No GitEHR Repository Detected</Title>
              <Text size="sm" c="dimmed" ta="center">
                Select a folder containing a GitEHR subject repository, or a Store root with gitehr-mpi.json.
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
            <TextInput
              label="First subject name"
              placeholder="A person, family member, or pet"
              value={firstSubjectName}
              onChange={(e) => setFirstSubjectName(e.currentTarget.value)}
              w="100%"
            />
            <Button
              size="md"
              variant="light"
              leftSection={<IconPlus size={18} />}
              onClick={handleCreateStore}
              loading={creating}
              fullWidth
            >
              Create New Store
            </Button>
          </Stack>
        </Card>
      </Center>
    );
  }

  if (!repoPath && storeRoot) {
    return (
      <AppShell
        className="app-shell"
        header={{ height: 64, offset: true }}
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
                  GitEHR Patient Index
                </Text>
                <Title order={4} className="brand-title">
                  Store Root
                </Title>
              </Box>
            </Group>
            <Group gap="sm">
                <TextInput
                placeholder="Search by subject, ID, NHS, MRN..."
                leftSection={<IconSearch size={16} />}
                size="sm"
                className="search-input"
                value={patientSearch}
                onChange={(e) => setPatientSearch(e.currentTarget.value)}
              />
              <TextInput
                placeholder="New subject name"
                size="sm"
                value={newSubjectName}
                onChange={(e) => setNewSubjectName(e.currentTarget.value)}
              />
              <Button
                variant="light"
                leftSection={<IconPlus size={18} />}
                onClick={handleAddSubject}
                loading={creating}
              >
                Add Subject
              </Button>
            </Group>
          </Group>
        </AppShell.Header>
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
                  <Title order={2}>Patient Index</Title>
                  <Text size="sm" c="dimmed">
                    Search and select a subject from this Store.
                  </Text>
                </Box>
                <Badge variant="light" color="teal">
                  {mpi ? mpi.patients.length : 0} subjects
                </Badge>
              </Group>

              <Card radius="md" className="panel-card">
                {mpiLoading ? (
                  <Center py="md">
                    <Loader size="sm" />
                  </Center>
                ) : filteredPatients.length === 0 ? (
                  <Text size="sm" c="dimmed" ta="center" py="md">
                    No subjects found. Add a subject to get started.
                  </Text>
                ) : (
                  <Stack gap="sm">
                    {filteredPatients.map((patient) => (
                      <Card key={patient.patient_id} withBorder padding="md" radius="md">
                        <Group justify="space-between" align="flex-start">
                          <Box>
                            <Text fw={600}>
                              {patient.repo_path.split(/[\\/]/).filter(Boolean).pop()}
                            </Text>
                            <Text size="xs" c="dimmed">
                              {patient.patient_id} · Updated {new Date(patient.updated_at).toLocaleString()}
                            </Text>
                            <Group gap="xs" mt="xs">
                              {patient.identifiers.slice(0, 4).map((id, i) => (
                                <Badge key={`${id.type}-${id.value}-${i}`} variant="light">
                                  {id.type}: {id.value}
                                </Badge>
                              ))}
                              {patient.identifiers.length === 0 && (
                                <Badge variant="light" color="gray">
                                  No identifiers
                                </Badge>
                              )}
                            </Group>
                          </Box>
                          <Button
                            size="sm"
                            variant="light"
                            onClick={() => setRepoPath(patient.repo_path)}
                          >
                            Open
                          </Button>
                        </Group>
                      </Card>
                    ))}
                  </Stack>
                )}
              </Card>
            </Stack>
          </Box>
        </AppShell.Main>
      </AppShell>
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
                {selectedSubjectName}
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
          {storeRoot && (
            <NavLink
            label="Patient Index"
              leftSection={<IconUser size={18} />}
              onClick={() => setRepoPath(null)}
            />
          )}
          <NavLink
            label="Overview"
            leftSection={<IconUser size={18} />}
            active
          />
          <NavLink label="Appointments" leftSection={<IconCalendar size={18} />} />
          <NavLink label="Reports" leftSection={<IconChartBar size={18} />} />
          <NavLink label="Journal" leftSection={<IconFileText size={18} />} />
          <NavLink label="Vitals" leftSection={<IconActivity size={18} />} />
          <Divider />
          <NavLink label="Settings" leftSection={<IconSettings size={18} />} />
          <Card className="sidebar-card" radius="md" mt="md">
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
                Timeline and summaries from {selectedSubjectName}.
              </Text>
            </Box>
            <Group gap="xs">
              <Badge size="lg" variant="light" color="teal">
                Active
              </Badge>
              <Badge size="lg" variant="light" color="gray">
                {selectedPatient?.patient_id.slice(0, 12) || ".gitehr"}
              </Badge>
            </Group>
          </Group>

          <Card radius="md" className="panel-card">
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
          <Card radius="md" className="panel-card">
            <Group>
              <Avatar radius="xl" size="lg" color="teal">
                {selectedSubjectName.slice(0, 2).toUpperCase()}
              </Avatar>
              <Box>
                <Text fw={600}>{selectedSubjectName}</Text>
                <Text size="sm" c="dimmed">
                  {repoPath}
                </Text>
              </Box>
            </Group>
            <Group gap="xs" mt="md">
              <Badge variant="light">Store-first</Badge>
              <Badge variant="light">Plain files</Badge>
              <Badge variant="light">Git-backed</Badge>
            </Group>
          </Card>

          <Card radius="md" className="panel-card">
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

            <Card radius="md" className="panel-card">
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
