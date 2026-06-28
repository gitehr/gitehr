import {
  AppShell,
  Badge,
  Box,
  Button,
  Card,
  Center,
  Divider,
  Group,
  Loader,
  Stack,
  Text,
  Textarea,
  TextInput,
  ThemeIcon,
  Title,
  Alert,
} from "@mantine/core";
import {
  IconAlertCircle,
  IconFileText,
  IconFolderOpen,
  IconSearch,
  IconPlus,
} from "@tabler/icons-react";
import { useEffect, useState } from "react";
import gitehrLogo from "./assets/gitehr-logo.svg";
import "./App.css";
import {
  addJournalEntry,
  getConfiguredStore,
  getCurrentDir,
  getJournalEntries,
  getStateFile,
  isGitehrRepo,
  hasMpi,
  getMpi,
  pickFolder,
  initStoreRoot,
  addStoreSubject,
  type JournalEntryInfo,
  type MpiInfo,
  type MpiPatientInfo,
} from "./api/gitehr";

interface PatientDemographics {
  title?: string;
  fullName?: string;
  preferredName?: string;
  address?: string;
  dateOfBirth?: string;
  nhsNumber?: string;
}

const DEMOGRAPHICS_FILES = ["patient.md", "demographics.md", "patient.json", "demographics.json"];

function parsePatientDemographics(filename: string, content: string): PatientDemographics {
  const data = filename.endsWith(".json")
    ? parseJsonObject(content)
    : parseYamlLikeObject(extractFrontMatter(content));

  const fullName =
    readField(data, ["patient_name", "full_name", "name"]) ||
    [readField(data, ["given_name", "first_name"]), readField(data, ["family_name", "surname"])]
      .filter(Boolean)
      .join(" ");

  return {
    title: readField(data, ["title", "honorific", "prefix"]),
    fullName: fullName || undefined,
    preferredName: readField(data, ["preferred_name", "known_as"]),
    address: readField(data, ["address", "home_address"]),
    dateOfBirth: readField(data, ["date_of_birth", "dob", "birth_date"]),
    nhsNumber: readField(data, ["nhs_number", "nhs"]),
  };
}

function parseJsonObject(content: string): Record<string, unknown> {
  try {
    const parsed = JSON.parse(content);
    return parsed && typeof parsed === "object" && !Array.isArray(parsed) ? parsed : {};
  } catch {
    return {};
  }
}

function extractFrontMatter(content: string): string {
  const match = content.match(/^---\s*\n([\s\S]*?)\n---/);
  return match ? match[1] : content;
}

function parseYamlLikeObject(content: string): Record<string, string> {
  const result: Record<string, string> = {};
  const lines = content.split(/\r?\n/);

  for (let i = 0; i < lines.length; i += 1) {
    const line = lines[i];
    const match = line.match(/^([A-Za-z0-9_-]+):\s*(.*)$/);
    if (!match) continue;

    const key = match[1];
    const value = match[2].trim();
    if (value) {
      result[key] = unquote(value);
      continue;
    }

    const block: string[] = [];
    while (i + 1 < lines.length && /^\s+/.test(lines[i + 1])) {
      i += 1;
      const item = lines[i].trim().replace(/^-\s*/, "");
      if (item) block.push(unquote(item));
    }
    if (block.length > 0) {
      result[key] = block.join(", ");
    }
  }

  return result;
}

function unquote(value: string): string {
  return value.replace(/^["']|["']$/g, "");
}

function readField(data: Record<string, unknown>, keys: string[]): string | undefined {
  for (const key of keys) {
    const value = data[key];
    if (typeof value === "string" && value.trim()) return value.trim();
    if (Array.isArray(value)) {
      const text = value.filter((item) => typeof item === "string" && item.trim()).join(", ");
      if (text) return text;
    }
  }
  return undefined;
}

function formatDate(value?: string): string | undefined {
  if (!value) return undefined;
  const date = new Date(value);
  if (Number.isNaN(date.getTime())) return value;
  return date.toLocaleDateString(undefined, {
    day: "2-digit",
    month: "short",
    year: "numeric",
  });
}

function App() {
  const [repoPath, setRepoPath] = useState<string | null>(null);
  const [storeRoot, setStoreRoot] = useState<string | null>(null);
  const [repoChecked, setRepoChecked] = useState(false);
  const [entries, setEntries] = useState<JournalEntryInfo[]>([]);
  const [demographics, setDemographics] = useState<PatientDemographics | null>(null);
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
          return;
        }

        const configuredStore = await getConfiguredStore();
        if (configuredStore) {
          const configuredHasIndex = await hasMpi(configuredStore);
          if (configuredHasIndex) {
            await loadMpi(configuredStore);
            return;
          }
          setError(
            `Configured Store path does not contain gitehr-mpi.json: ${configuredStore}`
          );
        }
      } catch (err) {
        console.error("Failed to check initial repo:", err);
        setError(typeof err === "string" ? err : String(err));
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

  const fetchDemographics = async (path: string): Promise<PatientDemographics | null> => {
    for (const filename of DEMOGRAPHICS_FILES) {
      try {
        const file = await getStateFile(path, filename);
        return parsePatientDemographics(file.name, file.content);
      } catch {
        // Demographics are optional today; try the next conventional filename.
      }
    }
    return null;
  };

  const fetchData = async () => {
    if (!repoPath) return;
    setLoading(true);
    setError(null);
    setDemographics(null);
    try {
      const [entriesData, demographicsData] = await Promise.all([
        getJournalEntries(repoPath, { limit: 10, reverse: true }),
        fetchDemographics(repoPath),
      ]);
      setEntries(entriesData);
      setDemographics(demographicsData);
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
  const displayName =
    demographics?.preferredName || demographics?.fullName || selectedSubjectName;
  const identifiers = [...(selectedPatient?.identifiers || [])];
  if (
    demographics?.nhsNumber &&
    !identifiers.some(
      (id) =>
        id.value === demographics.nhsNumber ||
        id.type.toLowerCase() === "nhs"
    )
  ) {
    identifiers.unshift({ type: "NHS", value: demographics.nhsNumber });
  }

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
      header={{ height: 104, offset: true }}
      navbar={{ width: 260, breakpoint: "sm" }}
      aside={{ width: 320, breakpoint: "md" }}
      padding="md"
    >
      <AppShell.Header className="app-header record-header">
        <Box className="record-header-inner">
          <Group gap="sm" className="record-header-brand" wrap="nowrap">
            <Box className="brand-mark">
              <img src={gitehrLogo} alt="GitEHR logo" />
            </Box>
            <Box>
              <Text size="xs" c="dimmed" fw={600} tt="uppercase">
                GitEHR Reference GUI
              </Text>
              <Title order={4} className="brand-title">
                {displayName}
              </Title>
            </Box>
          </Group>
          <Box className="patient-info-bar patient-info-bar-header">
            <Box className="patient-info-primary">
              <Text size="xs" c="dimmed" fw={600} tt="uppercase">
                Patient
              </Text>
              <Text fw={700} size="lg">
                {displayName}
              </Text>
            </Box>
            <Box className="patient-info-field">
              <Text size="xs" c="dimmed" fw={600} tt="uppercase">
                Title
              </Text>
              <Text size="sm">{demographics?.title || "Not recorded"}</Text>
            </Box>
            <Box className="patient-info-field">
              <Text size="xs" c="dimmed" fw={600} tt="uppercase">
                DOB
              </Text>
              <Text size="sm">{formatDate(demographics?.dateOfBirth) || "Not recorded"}</Text>
            </Box>
            <Box className="patient-info-field patient-info-address">
              <Text size="xs" c="dimmed" fw={600} tt="uppercase">
                Address
              </Text>
              <Text size="sm" lineClamp={2}>
                {demographics?.address || "Not recorded"}
              </Text>
            </Box>
            <Box className="patient-info-field patient-info-identifiers">
              <Text size="xs" c="dimmed" fw={600} tt="uppercase">
                Identifiers
              </Text>
              <Group gap={6}>
                {identifiers.length > 0 ? (
                  identifiers.slice(0, 4).map((id, i) => (
                    <Badge key={`${id.type}-${id.value}-${i}`} variant="light" color="gray">
                      {id.type}: {id.value}
                    </Badge>
                  ))
                ) : (
                  <Text size="sm">Not recorded</Text>
                )}
              </Group>
            </Box>
          </Box>
        </Box>
      </AppShell.Header>

      <AppShell.Navbar className="app-sidebar" />

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
                <Title order={2}>Journal</Title>
                <Text size="sm" c="dimmed">
                  Add and review journal entries for {displayName}.
                </Text>
              </Box>
              <Badge size="lg" variant="light" color="gray">
                {selectedPatient?.patient_id.slice(0, 12) || ".gitehr"}
              </Badge>
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

      <AppShell.Aside className="app-aside" />
    </AppShell>
  );
}

export default App;
