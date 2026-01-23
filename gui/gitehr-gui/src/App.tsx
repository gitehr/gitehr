import {
  AppShell,
  Avatar,
  Badge,
  Box,
  Button,
  Card,
  Divider,
  Group,
  List,
  NavLink,
  Progress,
  SimpleGrid,
  Stack,
  Text,
  TextInput,
  ThemeIcon,
  Title,
} from "@mantine/core";
import {
  IconActivity,
  IconCalendar,
  IconChartBar,
  IconClipboardHeart,
  IconFileText,
  IconMapPin,
  IconMessagePlus,
  IconSearch,
  IconSettings,
  IconStethoscope,
  IconUser,
} from "@tabler/icons-react";
import gitehrLogo from "./assets/gitehr-logo.svg";
import "./App.css";

function App() {
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
            <Button
              size="sm"
              leftSection={<IconMessagePlus size={16} />}
              radius="md"
            >
              New Entry
            </Button>
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
                Snapshot
              </Text>
              <Group justify="space-between">
                <Text size="sm">Active patients</Text>
                <Badge variant="light" color="teal">
                  32
                </Badge>
              </Group>
              <Group justify="space-between">
                <Text size="sm">Alerts</Text>
                <Badge variant="light" color="orange">
                  4
                </Badge>
              </Group>
            </Stack>
          </Card>
        </Stack>
      </AppShell.Navbar>

      <AppShell.Main className="app-main">
        <Box className="main-surface">
          <Stack gap="md">
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

          <SimpleGrid cols={{ base: 1, md: 2 }} spacing="md">
            <Card radius="lg" className="panel-card">
              <Group justify="space-between" mb="md">
                <Group gap="xs">
                  <ThemeIcon variant="light" color="teal">
                    <IconClipboardHeart size={18} />
                  </ThemeIcon>
                  <Text fw={600}>Care timeline</Text>
                </Group>
                <Badge variant="outline">Last 30 days</Badge>
              </Group>
              <List spacing="sm" size="sm">
                <List.Item icon={<IconStethoscope size={16} />}>
                  Follow-up visit for post-op assessment
                </List.Item>
                <List.Item icon={<IconFileText size={16} />}>
                  New journal entry: Cardio rehab plan v2
                </List.Item>
                <List.Item icon={<IconCalendar size={16} />}>
                  Remote consult scheduled for Thursday
                </List.Item>
                <List.Item icon={<IconMapPin size={16} />}>
                  Imaging uploaded from Outreach Lab
                </List.Item>
              </List>
            </Card>

            <Card radius="lg" className="panel-card">
              <Group justify="space-between" mb="md">
                <Group gap="xs">
                  <ThemeIcon variant="light" color="orange">
                    <IconActivity size={18} />
                  </ThemeIcon>
                  <Text fw={600}>Clinical summary</Text>
                </Group>
                <Badge variant="outline" color="orange">
                  Auto-generated
                </Badge>
              </Group>
              <Stack gap="sm">
                <Box>
                  <Group justify="space-between" mb={6}>
                    <Text size="sm">Medication adherence</Text>
                    <Text size="sm" c="dimmed">
                      92%
                    </Text>
                  </Group>
                  <Progress value={92} color="teal" />
                </Box>
                <Box>
                  <Group justify="space-between" mb={6}>
                    <Text size="sm">Recovery goals</Text>
                    <Text size="sm" c="dimmed">
                      64%
                    </Text>
                  </Group>
                  <Progress value={64} color="orange" />
                </Box>
                <Box>
                  <Group justify="space-between" mb={6}>
                    <Text size="sm">Vitals stability</Text>
                    <Text size="sm" c="dimmed">
                      78%
                    </Text>
                  </Group>
                  <Progress value={78} color="blue" />
                </Box>
              </Stack>
            </Card>
          </SimpleGrid>

          <SimpleGrid cols={{ base: 1, md: 3 }} spacing="md">
            <Card radius="lg" className="panel-card">
              <Text fw={600} mb="xs">
                Recent notes
              </Text>
              <Text size="sm" c="dimmed">
                New allergy documented: acetaminophen sensitivity, mild reaction
                noted.
              </Text>
              <Divider my="sm" />
              <Text size="sm" c="dimmed">
                Physical therapy: range of motion improved by 12% over baseline.
              </Text>
            </Card>
            <Card radius="lg" className="panel-card">
              <Text fw={600} mb="xs">
                Outstanding tasks
              </Text>
              <List spacing="xs" size="sm">
                <List.Item>Confirm lab results delivery</List.Item>
                <List.Item>Schedule nurse check-in</List.Item>
                <List.Item>Review imaging delta</List.Item>
              </List>
            </Card>
            <Card radius="lg" className="panel-card">
              <Text fw={600} mb="xs">
                Alerts
              </Text>
              <Stack gap="xs">
                <Badge color="red" variant="light">
                  Medication refill required
                </Badge>
                <Badge color="yellow" variant="light">
                  Consent expiring in 14 days
                </Badge>
                <Badge color="teal" variant="light">
                  Sync healthy
                </Badge>
              </Stack>
            </Card>
          </SimpleGrid>
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
              <Box>
                <Text size="sm" fw={600}>
                  Allergies
                </Text>
                <Text size="sm" c="dimmed">
                  Penicillin, acetaminophen
                </Text>
              </Box>
              <Box>
                <Text size="sm" fw={600}>
                  Current medications
                </Text>
                <Text size="sm" c="dimmed">
                  Atenolol 25mg, Atorvastatin 40mg
                </Text>
              </Box>
              <Box>
                <Text size="sm" fw={600}>
                  Demographics
                </Text>
                <Text size="sm" c="dimmed">
                  London, UK · Preferred pronouns: they/them
                </Text>
              </Box>
            </Stack>
          </Card>

          <Card radius="lg" className="panel-card">
            <Text size="xs" tt="uppercase" fw={600} c="dimmed" mb="xs">
              Activity feed
            </Text>
            <Stack gap="sm">
              <Group align="flex-start">
                <ThemeIcon variant="light" color="teal">
                  <IconFileText size={16} />
                </ThemeIcon>
                <Box>
                  <Text size="sm">Journal entry signed by Dr. Malik</Text>
                  <Text size="xs" c="dimmed">
                    18 minutes ago
                  </Text>
                </Box>
              </Group>
              <Group align="flex-start">
                <ThemeIcon variant="light" color="blue">
                  <IconCalendar size={16} />
                </ThemeIcon>
                <Box>
                  <Text size="sm">Appointment updated to telehealth</Text>
                  <Text size="xs" c="dimmed">
                    2 hours ago
                  </Text>
                </Box>
              </Group>
            </Stack>
          </Card>
        </Stack>
      </AppShell.Aside>
    </AppShell>
  );
}

export default App;
